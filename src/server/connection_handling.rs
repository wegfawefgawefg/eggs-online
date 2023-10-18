use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{
        atomic::{AtomicBool, AtomicU32},
        Arc,
    },
};

use crossbeam::queue::ArrayQueue;
use lazy_static::lazy_static;
use tokio::{
    io::{self},
    net::UdpSocket,
    sync::RwLock,
};

use super::settings::SERVER_ADDR;
use crate::common::{
    client_to_server::{ClientToServerMessage, ClientToServerMessageBundle},
    server_to_client::ServerToClientMessage,
};

pub type ClientMessageQueue = Arc<ArrayQueue<ServerToClientMessage>>;

lazy_static! {
    pub static ref INCOMING_MESSAGE_QUEUE: Arc<ArrayQueue<ClientToServerMessageBundle>> =
        Arc::new(ArrayQueue::new(1000));
    pub static ref NEXT_CONNECTION_ID: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref CLIENT_OUTBOUND_MAILBOXES: RwLock<HashMap<u32, ClientMessageQueue>> =
        RwLock::new(HashMap::new());
    pub static ref CLIENT_DISCONNECTED: Arc<RwLock<HashMap<u32, Arc<AtomicBool>>>> =
        Arc::new(RwLock::new(HashMap::new()));
    pub static ref CLIENT_ID_TO_SOCKET_ADDRESS: Arc<RwLock<HashMap<u32, SocketAddr>>> =
        Arc::new(RwLock::new(HashMap::new()));
    pub static ref SOCKET_ADDRESS_TO_CLIENT_ID: Arc<RwLock<HashMap<SocketAddr, u32>>> =
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
    // let id = add_client().await;
    // socket.write_all(&id.to_be_bytes()).await?;

    println!("Listening for incoming messages...");
    let mut buffer = [0; 1024];
    loop {
        // println!("waiting for message...");
        let (nbytes, socket_address) = socket.recv_from(&mut buffer).await?;
        // println!("Received {} bytes from {}", nbytes, socket_address);

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
        // tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

////////////////////////    CLIENT BOOKKEEPING    ////////////////////////
pub fn get_next_connection_id() -> u32 {
    NEXT_CONNECTION_ID.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
}

pub async fn add_client(socket_address: SocketAddr) -> u32 {
    let id = get_next_connection_id();

    let mailbox = Arc::new(ArrayQueue::new(100));

    // Insert into CLIENT_OUTBOUND_MAILBOXES
    {
        let mut clients_write = CLIENT_OUTBOUND_MAILBOXES.write().await;
        clients_write.insert(id, mailbox);
    }

    // Insert into CLIENT_DISCONNECTED flag map
    {
        let disconnected = Arc::new(AtomicBool::new(false));
        let mut client_status_write = CLIENT_DISCONNECTED.write().await;
        client_status_write.insert(id, disconnected.clone());
    }

    // Insert into CLIENT_SOCKET_ADDRESSES
    {
        let mut client_socket_addresses_write = CLIENT_ID_TO_SOCKET_ADDRESS.write().await;
        client_socket_addresses_write.insert(id, socket_address);
    }

    // Insert into SOCKET_ADDRESS_TO_CLIENT_ID
    {
        let mut socket_address_to_client_id_write = SOCKET_ADDRESS_TO_CLIENT_ID.write().await;
        socket_address_to_client_id_write.insert(socket_address, id);
    }

    // announce that theres a new connection
    {
        let to_self_message = ClientToServerMessageBundle {
            client_id: id,
            message: ClientToServerMessage::Connect,
        };
        if INCOMING_MESSAGE_QUEUE.push(to_self_message).is_err() {
            eprintln!(
                "Inbound message queue full: dropping disconnect message from {}",
                id
            );
        }
    }

    // tell client his id
    {
        let new_id_message = ServerToClientMessage::ClientIDAssignment { new_client_id: id };
        let client_outbound_mailboxes_read = CLIENT_OUTBOUND_MAILBOXES.read().await;
        if let Some(client_mailbox) = client_outbound_mailboxes_read.get(&id) {
            if client_mailbox.push(new_id_message).is_err() {
                eprintln!(
                    "Inbound message queue full: dropping disconnect message from {}",
                    id
                );
            }
        }
    }

    println!("New Connected {}. Assigned ID: {}", socket_address, id);
    id
}

///  Removes client allocated bookkeeping resources.
pub async fn remove_client(id: u32) {
    // Remove from CLIENT_OUTBOUND_MAILBOXES
    {
        let mut clients_write = CLIENT_OUTBOUND_MAILBOXES.write().await;
        clients_write.remove(&id);
    }

    // Remove from CLIENT_DISCONNECTED flag map
    {
        let mut client_status_write = CLIENT_DISCONNECTED.write().await;
        client_status_write.remove(&id);
    }

    // Remove from SOCKET_ADDRESS_TO_CLIENT_ID
    {
        // fetch id from SOCKET_ADDRESS_TO_CLIENT_ID
        let client_id_to_socket_address_read = CLIENT_ID_TO_SOCKET_ADDRESS.read().await;
        if let Some(socket_address) = client_id_to_socket_address_read.get(&id) {
            {
                let mut socket_address_to_client_id_write =
                    SOCKET_ADDRESS_TO_CLIENT_ID.write().await;
                socket_address_to_client_id_write.remove(socket_address);
            }
        } else {
            eprintln!("Failed to find socket address for client {}", id);
            return;
        }
    }

    // Remove from CLIENT_ID_TO_SOCKET_ADDRESS
    {
        let mut client_socket_addresses_write = CLIENT_ID_TO_SOCKET_ADDRESS.write().await;
        client_socket_addresses_write.remove(&id);
    }

    println!("Client {} network resources cleaned up.", id);
}

////////////////////////    ENQUEUE OUTBOUND MESSAGES    ////////////////////////
pub async fn send_to_one_client(client_id: u32, message: ServerToClientMessage) {
    let clients_read = CLIENT_OUTBOUND_MAILBOXES.read().await;
    if let Some(queue) = clients_read.get(&client_id) {
        if queue.push(message).is_err() {
            eprintln!("Failed to enqueue message for client {}", client_id);
        }
    } else {
        eprintln!("Failed to find client {}", client_id);
    }
}

pub async fn broadcast_to_all_except(sender_id: u32, message: ServerToClientMessage) {
    let clients_read = CLIENT_OUTBOUND_MAILBOXES.read().await;
    for (&client_id, queue) in clients_read.iter() {
        if client_id == sender_id {
            continue; // Skip the sender
        }
        if queue.push(message.clone()).is_err() {
            eprintln!("Failed to enqueue message for client {}", client_id);
        }
    }
}

pub async fn broadcast_to_all(message: ServerToClientMessage) {
    let clients_read = CLIENT_OUTBOUND_MAILBOXES.read().await;
    for (_, queue) in clients_read.iter() {
        if queue.push(message.clone()).is_err() {
            eprintln!("Failed to enqueue message for client");
        }
    }
}
