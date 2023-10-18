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
use tokio::sync::RwLock;

use crate::{
    common::{
        client_to_server::{ClientToServerMessage, ClientToServerMessageBundle},
        server_to_client::ServerToClientMessage,
    },
    server::udp_networking::{CLIENT_DISCONNECTED, INCOMING_MESSAGE_QUEUE},
};

pub type ClientMessageQueue = Arc<ArrayQueue<ServerToClientMessage>>;

lazy_static! {
    pub static ref NEXT_CONNECTION_ID: Arc<AtomicU32> = Arc::new(AtomicU32::new(0));
    pub static ref CLIENT_ID_TO_SOCKET_ADDRESS: Arc<RwLock<HashMap<u32, SocketAddr>>> =
        Arc::new(RwLock::new(HashMap::new()));
    pub static ref SOCKET_ADDRESS_TO_CLIENT_ID: Arc<RwLock<HashMap<SocketAddr, u32>>> =
        Arc::new(RwLock::new(HashMap::new()));
    pub static ref CLIENT_OUTBOUND_MAILBOXES: RwLock<HashMap<u32, ClientMessageQueue>> =
        RwLock::new(HashMap::new());
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
