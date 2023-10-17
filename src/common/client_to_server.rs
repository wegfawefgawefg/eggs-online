use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientToServerMessage {
    Connect,
    Disconnect,
    ChatMessage { message: String },
    RequestToSpawnPlayer,
    RequestAllPlayers,
    EntityPosition { entity_id: u32, pos: glam::Vec2 },
}

pub struct ClientToServerMessageBundle {
    pub client_id: u32,
    pub message: ClientToServerMessage,
}
