use std::time::Instant;

use glam::Vec2;

use crate::{
    common::{
        client_to_server::ClientToServerMessage, game_objects::Player,
        server_to_client::ServerToClientMessage,
    },
    server::connection_handling::{broadcast_to_all, broadcast_to_all_except, send_to_one_client},
};

use super::{connection_handling::INCOMING_MESSAGE_QUEUE, state::State};

pub const FRAMES_PER_SECOND: u32 = 1;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

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
                let outbound_message = ServerToClientMessage::PlayerJoined { id: client_id };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
            ClientToServerMessage::Disconnect => {
                println!("Client {} disconnected", client_id);

                // announce the leave
                let outbound_message = ServerToClientMessage::PlayerLeft { id: client_id };
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

                // announce the spawn
                let outbound_message = ServerToClientMessage::SpawnPlayer {
                    owner_client_id: client_id,
                    entity_id: eid,
                    pos: Vec2::ZERO,
                };
                broadcast_to_all(outbound_message).await;
            }
            ClientToServerMessage::EntityPosition { entity_id, pos } => {
                println!("{} sent position: {:?}", client_id, pos);
                if let Some(player) = state.players.get_mut(&entity_id) {
                    player.pos = pos;
                }

                let outbound_message = ServerToClientMessage::EntityPosition { entity_id, pos };
                broadcast_to_all_except(client_id, outbound_message).await;
            }
        }
    }
}
