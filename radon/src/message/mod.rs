use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum MessageType {
    Join,
    Leave,
    Text,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextMessage {
    pub kind: MessageType,
    pub from: Option<String>,
    pub text: String,
}

impl TextMessage {
    pub fn new(kind: MessageType, from: Option<String>, text: String) -> Self {
        Self { kind, from, text }
    }
}
