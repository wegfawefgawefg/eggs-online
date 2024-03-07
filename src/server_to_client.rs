use glam::Vec2;
use serde::{Deserialize, Serialize};

use crate::game_objects::Player;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    ClientIDAssignment {
        new_client_id: u32,
    },
    Welcome {
        server_message: String,
    },
    ClientJoined {
        id: u32,
    },
    ClientLeft {
        id: u32,
    },
    ChatMessage {
        from: u32,
        message: String,
    },
    SpawnPlayer {
        owner_client_id: u32,
        entity_id: u32,
        pos: Vec2,
    },
    EntityPosition {
        entity_id: u32,
        pos: Vec2,
    },
    AllPlayers {
        players: Vec<Player>,
    },
}
