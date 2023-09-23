use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum MessageType {
    Register,
    Chat,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Message {
    pub message_type: MessageType,
    pub payload: String,
}

impl Message {
    pub fn new(message_type: MessageType, payload: String) -> Self {
        Self {
            message_type,
            payload,
        }
    }
}
