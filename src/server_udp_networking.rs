use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{atomic::AtomicBool, Arc},
};

use crossbeam::queue::ArrayQueue;
use lazy_static::lazy_static;
use tokio::{
    io::{self},
    net::UdpSocket,
    sync::RwLock,
};

use crate::{
    bookkeeping::{add_client, SOCKET_ADDRESS_TO_CLIENT_ID},
    client_to_server::{ClientToServerMessage, ClientToServerMessageBundle},
};
use crate::{
    bookkeeping::{CLIENT_ID_TO_SOCKET_ADDRESS, CLIENT_OUTBOUND_MAILBOXES},
    settings::SERVER_ADDR,
};

lazy_static! {
    pub static ref INCOMING_MESSAGE_QUEUE: Arc<ArrayQueue<ClientToServerMessageBundle>> =
        Arc::new(ArrayQueue::new(32));
    pub static ref CLIENT_DISCONNECTED: Arc<RwLock<HashMap<u32, Arc<AtomicBool>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

////////////////////////    CLIENT RX/TX TASKS    ////////////////////////

pub async fn init() -> tokio::io::Result<()> {
    println!("Initializing socket...");
    let socket = Arc::new(UdpSocket::bind(SERVER_ADDR).await.unwrap());
    println!("Socket Initialized!");
    println!("Spawning rx/tx tasks...");
    tokio::spawn(continuously_read_any_inbound_messages(socket.clone()));
    tokio::spawn(continuously_transmit_any_outbound_messages(socket.clone()));
    Ok(())
}

pub async fn continuously_read_any_inbound_messages(socket: Arc<UdpSocket>) -> io::Result<()> {
    println!("Listening for incoming messages...");
    let mut buffer = [0; 1024];
    loop {
        let (nbytes, socket_address) = socket.recv_from(&mut buffer).await?;

        // check if new client
        let maybe_client_id: Option<u32> = {
            let socket_address_to_client_id_read = SOCKET_ADDRESS_TO_CLIENT_ID.read().await;
            socket_address_to_client_id_read
                .get(&socket_address)
                .copied()
        };
        let client_id = match maybe_client_id {
            Some(client_id) => client_id,
            None => add_client(socket_address).await,
        };

        let result: Result<ClientToServerMessage, _> = bincode::deserialize(&buffer[..nbytes]);
        match result {
            Ok(result) => {
                let message_bundle = ClientToServerMessageBundle {
                    client_id,
                    message: result,
                };
                if INCOMING_MESSAGE_QUEUE.push(message_bundle).is_err() {
                    eprintln!(
                        "Inbound message queue full: dropping message from {}",
                        client_id
                    );
                }
            }
            Err(e) => {
                eprintln!("Error parsing client data: {:?}", e);
            }
        }
        // tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}

pub async fn continuously_transmit_any_outbound_messages(socket: Arc<UdpSocket>) -> io::Result<()> {
    // transmit any outbound messages
    loop {
        // loop through every mailbox
        let clients_read = CLIENT_OUTBOUND_MAILBOXES.read().await;
        for (&client_id, queue) in clients_read.iter() {
            // is there a socket for this client?
            let maybe_socket_address: Option<SocketAddr> = {
                let client_id_to_socket_address_read = CLIENT_ID_TO_SOCKET_ADDRESS.read().await;
                client_id_to_socket_address_read.get(&client_id).copied()
            };

            if maybe_socket_address.is_none() {
                eprintln!("Failed to find socket address for client {}", client_id);
                continue;
            }

            // if yes, send his messages
            if let Some(socket_address) = maybe_socket_address {
                // send messages if theres a registered socket for this client
                const MAX_MESSAGES_PER_CLIENT_FRAME: usize = 128;
                let mut messages_sent_this_client = 0;
                while let Some(message) = queue.pop() {
                    // dont let one noisy client clog up message processing
                    if messages_sent_this_client >= MAX_MESSAGES_PER_CLIENT_FRAME {
                        break;
                    }
                    match bincode::serialize(&message) {
                        Ok(binary_message) => {
                            socket.send_to(&binary_message, socket_address).await?;
                        }
                        Err(e) => {
                            eprintln!("Error serializing message: {:?}", e);
                        }
                    }
                    messages_sent_this_client += 1;
                }
            }
        }
        // tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
    }
}
