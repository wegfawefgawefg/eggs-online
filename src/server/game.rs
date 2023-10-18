use std::{collections::HashMap, time::Instant};

use glam::Vec2;

use crate::{
    common::{
        client_to_server::{ClientToServerMessage, ClientToServerMessageBundle},
        game_objects::Player,
        server_to_client::ServerToClientMessage,
    },
    server::enque_outbound_messages::{
        broadcast_to_all, broadcast_to_all_except, send_to_one_client,
    },
};

use super::{state::State, udp_networking::INCOMING_MESSAGE_QUEUE};

pub const FRAMES_PER_SECOND: u32 = 60;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

pub const DEBUG_PRINT_PROCESSED_MESSAGES: bool = false;

pub async fn main_loop(state: &mut State) {
    let mut previous_time = Instant::now();
    loop {
        process_message_queue(state).await;

        let current_time = Instant::now();
        let dt = (current_time - previous_time).as_secs_f32();
        previous_time = current_time;

        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            step(state);
            // state.print_state();
        }
    }
}

pub fn step(state: &mut State) {
    for (_, player) in state.players.iter_mut() {
        player.step();
    }
    // state.print_state();
}

pub async fn process_message_queue(state: &mut State) {
    // prune_latest_only_messages().await;

    while let Some(message_bundle) = INCOMING_MESSAGE_QUEUE.pop() {
        let client_id = message_bundle.client_id;
        match message_bundle.message {
            ClientToServerMessage::Connect => {
                println!("Client {} connected", client_id);

                // send welcome
                let outbound_message = ServerToClientMessage::Welcome {
                    server_message: "welcome to the server".to_string(),
                };
                send_to_one_client(client_id, outbound_message).await;

                // announce the join
                let outbound_message = ServerToClientMessage::ClientJoined { id: client_id };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
            ClientToServerMessage::Disconnect => {
                println!("Client {} disconnected", client_id);

                // announce the leave
                let outbound_message = ServerToClientMessage::ClientLeft { id: client_id };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
            ClientToServerMessage::ChatMessage { message } => {
                println!("{} says: {}", client_id, message);

                // broadcast the message
                let outbound_message = ServerToClientMessage::ChatMessage {
                    from: client_id,
                    message,
                };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
            ClientToServerMessage::RequestToSpawnPlayer => {
                println!("{} requested to spawn a player", client_id);

                let eid = state.next_eid;
                state.next_eid += 1;

                // spawn the player
                let player = Player::new(client_id, eid);
                state.players.insert(eid, player);
                println!("spawned player {}", eid);

                // announce the spawn
                let outbound_message = ServerToClientMessage::SpawnPlayer {
                    owner_client_id: client_id,
                    entity_id: eid,
                    pos: Vec2::ZERO,
                };
                broadcast_to_all(outbound_message).await;
            }
            ClientToServerMessage::EntityPosition { entity_id, pos } => {
                if let Some(player) = state.players.get_mut(&entity_id) {
                    player.pos = pos;
                }

                let outbound_message = ServerToClientMessage::EntityPosition { entity_id, pos };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
            ClientToServerMessage::RequestAllPlayers => {
                println!("{} requested all players", client_id);

                let players = state.players.values().cloned().collect();

                let outbound_message = ServerToClientMessage::AllPlayers { players };
                send_to_one_client(client_id, outbound_message).await;
            }
        }
    }
}

pub async fn prune_latest_only_messages() {
    let queue = INCOMING_MESSAGE_QUEUE.clone();

    let mut num_pruned = 0;
    let mut latest_messages: HashMap<u8, ClientToServerMessageBundle> = HashMap::new();
    let mut to_retain: Vec<ClientToServerMessageBundle> = Vec::new();

    while let Some(bundle) = queue.pop() {
        if let Some(keep_latest_only_id) = get_keep_latest_only_message_type_id(&bundle.message) {
            latest_messages.insert(keep_latest_only_id, bundle.clone());
            num_pruned += 1;
        } else {
            to_retain.push(bundle);
        }
    }
    // println!("num pruned {}", num_pruned);

    // Add the latest messages back to the retained list
    to_retain.extend(latest_messages.values().cloned());

    // Refill the queue with the pruned list
    for bundle in to_retain.iter() {
        queue.push(bundle.clone()).unwrap();
    }
}

pub fn get_keep_latest_only_message_type_id(message: &ClientToServerMessage) -> Option<u8> {
    match message {
        ClientToServerMessage::EntityPosition { .. } => Some(0),
        _ => None,
    }
}
