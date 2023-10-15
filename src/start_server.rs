use server::connection_handling;

mod common;
mod server;

#[tokio::main]
async fn main() {
    connection_handling::init().await;

    let mut state = server::state::State::new();
    server::game::main_loop(&mut state).await;
}
