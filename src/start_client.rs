use {
    client_game::process_message_queue, client_to_server::ClientToServerMessage,
    event_processing::process_events_and_input, state::State,
};

mod bookkeeping;
mod client_game;
mod client_to_server;
mod client_udp_networking;
mod components;
mod draw;
mod enque_outbound_messages;
mod event_processing;
mod game_objects;
mod graphics;
mod server_game;
mod server_state;
mod server_to_client;
mod server_udp_networking;
mod settings;
mod state;

pub const FRAMES_PER_SECOND: u32 = 60;
const TIMESTEP: f32 = 1.0 / FRAMES_PER_SECOND as f32;

const POSITION_TRANSMIT_FREQUENCY: u32 = 1;

#[derive(PartialEq, Eq)]
enum Bool {
    True,
    False,
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    let result = client_udp_networking::init_connection().await;
    if let Err(e) = result {
        eprintln!("Error connecting to server: {:?}", e);
        return Ok(());
    }

    // request a new player
    if client_udp_networking::OUTBOUND_MESSAGE_QUEUE
        .push(ClientToServerMessage::RequestToSpawnPlayer)
        .is_err()
    {
        eprintln!("Outbound message queue full: dropping message");
    }

    // request all players
    if client_udp_networking::OUTBOUND_MESSAGE_QUEUE
        .push(ClientToServerMessage::RequestAllPlayers)
        .is_err()
    {
        eprintln!("Outbound message queue full: dropping message");
    }

    let (mut rl, mut rlt, mut render_texture) = graphics::init_graphics();

    ////////////////    MAIN LOOP    ////////////////
    let mut state = State::new();

    let mut position_transmit_counter = POSITION_TRANSMIT_FREQUENCY;

    while !rl.window_should_close() {
        process_events_and_input(&mut rl, &mut state);

        // state transmitting
        {
            position_transmit_counter -= 1;
            if position_transmit_counter == 0 {
                position_transmit_counter = POSITION_TRANSMIT_FREQUENCY;

                for player in state.players.values() {
                    if let Some(client_id) = state.client_id {
                        if player.owner_client_id == client_id
                            && client_udp_networking::OUTBOUND_MESSAGE_QUEUE
                                .push(ClientToServerMessage::EntityPosition {
                                    entity_id: player.entity_id,
                                    pos: player.pos,
                                })
                                .is_err()
                        {
                            eprintln!("Outbound message queue full: dropping message");
                        }
                    }
                }
            }
        }

        process_message_queue(&mut state).await;

        let dt = rl.get_frame_time();
        state.time_since_last_update += dt;
        while state.time_since_last_update > TIMESTEP {
            state.time_since_last_update -= TIMESTEP;

            client_game::step(&mut state);
        }

        graphics::render(&mut rl, &mut rlt, &mut render_texture, &state);

        if !state.running {
            break;
        }
    }
    Ok(())
}
