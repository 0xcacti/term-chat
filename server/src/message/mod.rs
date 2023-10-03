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

    pub fn encode(&self) -> Vec<u8> {
        let message_json = serde_json::to_string(&self).unwrap();
        let message_len = message_json.len() as u32;
        let message_len_buf = message_len.to_be_bytes().to_vec();
        let mut message_buf = message_len_buf;
        message_buf.extend_from_slice(message_json.as_bytes());
        message_buf
    }
}
