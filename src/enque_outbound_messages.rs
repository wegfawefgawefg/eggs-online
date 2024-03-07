use crate::server_to_client::ServerToClientMessage;

use crate::bookkeeping::CLIENT_OUTBOUND_MAILBOXES;

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
