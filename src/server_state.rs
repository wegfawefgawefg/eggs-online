use std::{collections::HashMap, sync::Arc};

use tokio::{net::TcpStream, sync::Mutex};

use crate::game_objects::Player;

pub struct ServerState {
    pub time_since_last_update: f32,
    pub next_id: u32,
    pub next_eid: u32,
    pub players: HashMap<u32, Player>,
    pub clients: HashMap<u32, Arc<Mutex<TcpStream>>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            time_since_last_update: 0.0,
            next_id: 0,
            next_eid: 0,
            players: HashMap::new(),
            clients: HashMap::new(),
        }
    }

    // pub fn print_state(&self) {
    //     for (id, player) in self.players.iter() {
    //         println!("Player {}: pos: {}, vel: {}", id, player.pos, player.vel);
    //     }
    // }
}

impl Default for ServerState {
    fn default() -> Self {
        Self::new()
    }
}
