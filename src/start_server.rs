mod common;
mod server;

#[tokio::main]
async fn main() {
    let _ = server::udp_networking::init().await;

    let mut state = server::state::State::new();
    server::game::main_loop(&mut state).await;
}
