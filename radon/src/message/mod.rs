use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
    pub from: String,
    pub text: String,
}

impl Message {
    pub fn new(from: String, text: String) -> Self {
        Self { from, text }
    }
}
