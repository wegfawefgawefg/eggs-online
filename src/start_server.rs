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

#[tokio::main]
async fn main() {
    let _ = server_udp_networking::init().await;

    let mut state = server_state::ServerState::new();
    server_game::main_loop(&mut state).await;
}
