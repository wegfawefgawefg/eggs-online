use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerToClientMessage {
    Welcome { server_message: String },
    PlayerJoined { id: u32 },
    PlayerLeft { id: u32 },
    ChatMessage { from: u32, message: String },
}
