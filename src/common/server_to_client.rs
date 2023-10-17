use glam::Vec2;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    Welcome {
        server_message: String,
    },
    PlayerJoined {
        id: u32,
    },
    PlayerLeft {
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
}
