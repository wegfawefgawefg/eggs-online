use common::client_to_server::ClientToServerMessage;
use raylib::prelude::*;

use client::{
    event_processing::process_events_and_input,
    game::process_message_queue,
    graphics::{scale_and_blit_render_texture_to_window, FULLSCREEN, WINDOW_DIMS},
    state::State,
};
mod client;
mod common;
mod server;

pub const FRAMES_PER_SECOND: u32 = 60;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

const POSITION_TRANSMIT_FREQUENCY: u32 = 16;

#[derive(PartialEq, Eq)]
enum Bool {
    True,
    False,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let result = client::connection_handling::init_connection().await;
    if let Err(e) = result {
        eprintln!("Error connecting to server: {:?}", e);
        return Ok(());
    }

    // request a new player
    if client::connection_handling::OUTBOUND_MESSAGE_QUEUE
        .push(ClientToServerMessage::RequestToSpawnPlayer)
        .is_err()
    {
        eprintln!("Outbound message queue full: dropping message");
    }

    // request all players
    if client::connection_handling::OUTBOUND_MESSAGE_QUEUE
        .push(ClientToServerMessage::RequestAllPlayers)
        .is_err()
    {
        eprintln!("Outbound message queue full: dropping message");
    }

    let (mut rl, mut rlt, mut render_texture) = client::graphics::init_graphics();

    ////////////////    MAIN LOOP    ////////////////
    let mut state = client::state::State::new();

    let mut position_transmit_counter = POSITION_TRANSMIT_FREQUENCY;
    let mut player_last_pos = None;

    while !rl.window_should_close() {
        process_events_and_input(&mut rl, &mut state);

        // state transmitting
        position_transmit_counter -= 1;
        if position_transmit_counter == 0 {
            position_transmit_counter = POSITION_TRANSMIT_FREQUENCY;
            if let Some(player_id) = state.player_id {
                if let Some(our_player) = state.players.get(&player_id) {
                    if let Some(last_pos) = player_last_pos {
                        if last_pos == our_player.pos {
                            // println!("Not sending position: {:?}", our_player.pos);
                            continue;
                        }
                    }
                    player_last_pos = Some(our_player.pos);

                    // println!("Sending position: {:?}", our_player.pos);
                    if client::connection_handling::OUTBOUND_MESSAGE_QUEUE
                        .push(ClientToServerMessage::EntityPosition {
                            entity_id: player_id,
                            pos: our_player.pos,
                        })
                        .is_err()
                    {
                        eprintln!("Outbound message queue full: dropping message");
                    }
                }
            }
        }

        process_message_queue(&mut state).await;

        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            client::game::step(&mut state);
        }

        client::graphics::render(&mut rl, &mut rlt, &mut render_texture, &state);

        if !state.running {
            break;
        }
    }
    Ok(())
}
