use std::sync::atomic::Ordering;

use glam::Vec2;

use crate::common::{game_objects::Player, server_to_client::ServerToClientMessage};

use super::{
    connection_handling::{CLIENT_ID, INCOMING_MESSAGE_QUEUE},
    state::State,
};

pub fn step(state: &mut State) {
    for (_, player) in state.players.iter_mut() {
        player.step();
    }
}

// use std::time::Instant;

// use crate::{
//     common::{client_to_server::ClientToServerMessage, server_to_client::ServerToClientMessage},
//     server::connection_handling::{broadcast_to_all_except, send_to_one_client},
// };

// use super::{connection_handling::INCOMING_MESSAGE_QUEUE, state::State};

// pub const FRAMES_PER_SECOND: u32 = 1;
// const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

// pub async fn main_loop(state: &mut State) {
//     let mut previous_time = Instant::now();
//     loop {
//

//         let current_time = Instant::now();
//         let dt = (current_time - previous_time).as_secs_f32();
//         previous_time = current_time;

//         state.time_since_last_update += dt;
//         while state.time_since_last_update > TIMESTEP {
//             state.time_since_last_update -= TIMESTEP;

//             step(state);
//         }
//     }
// }

pub async fn process_message_queue(state: &mut State) {
    while let Some(message) = INCOMING_MESSAGE_QUEUE.pop() {
        match message {
            ServerToClientMessage::Welcome { server_message } => {
                println!("Server says: {}", server_message);
            }
            ServerToClientMessage::PlayerJoined { id } => {
                println!("Player {} joined", id);
            }
            ServerToClientMessage::PlayerLeft { id } => {
                println!("Player {} left", id);
            }
            ServerToClientMessage::ChatMessage { from, message } => {
                println!("{} says: {}", from, message);
            }
            ServerToClientMessage::SpawnPlayer {
                owner_client_id,
                entity_id,
                pos,
            } => {
                // if the owner_client_id is our id, set out state.player_id to Some(owner_client_id)
                let our_client_id = CLIENT_ID.load(Ordering::SeqCst);
                if our_client_id == owner_client_id {
                    state.player_id = Some(entity_id);
                }

                state.players.insert(
                    entity_id,
                    Player {
                        owner_client_id,
                        id: entity_id,
                        pos,
                        vel: Vec2::new(0.0, 0.0),
                    },
                );
            }
            ServerToClientMessage::EntityPosition { entity_id, pos } => {
                if let Some(player) = state.players.get_mut(&entity_id) {
                    player.pos = pos;
                }
            }
        }
    }
}

// pub fn process_message_queue() {
//     while let Some(message) = INCOMING_MESSAGE_QUEUE.pop() {
//         match message {
//             ServerToClientMessage::Welcome { server_message } => {
//                 println!("Server says: {}", server_message);
//             }
//             ServerToClientMessage::PlayerJoined { id } => {
//                 println!("Player {} joined", id);
//             }
//             ServerToClientMessage::PlayerLeft { id } => {
//                 println!("Player {} left", id);
//             }
//             ServerToClientMessage::ChatMessage { from, message } => {
//                 println!("{} says: {}", from, message);
//             }
//         }
//     }
// }
