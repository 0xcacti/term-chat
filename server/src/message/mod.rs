use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum MessageType {
    Register,
    Chat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
}
