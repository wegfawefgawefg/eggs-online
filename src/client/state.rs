use std::collections::HashMap;

use crate::common::game_objects::Player;

pub struct State {
    pub running: bool,
    pub time_since_last_update: f32,
    pub client_id: Option<u32>,
    pub players: HashMap<u32, Player>,
}

impl State {
    pub fn new() -> Self {
        Self {
            running: true,
            time_since_last_update: 0.0,
            client_id: None,
            players: HashMap::new(),
        }
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}
