pub mod message;

use anyhow::Result;
use log::error;
use message::{Message, MessageType};
use serde::Deserialize;
use tokio::{
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            username: Some(String::new()),
            password: Some(String::new()),
        }
    }
}

impl Config {
    pub fn new(username: String, password: String) -> Self {
        Self {
            username: Some(username),
            password: Some(password),
        }
    }
}

pub async fn connect() -> Result<()> {
    let server_address = "127.0.0.1:8080";
    let socket = TcpStream::connect(server_address).await?;
    println!("Connected to server at {}", server_address);
    let (mut reader, mut writer) = socket.into_split();
    let read_task = tokio::spawn(async move {
        loop {
            let mut len_buf = [0u8; 4];
            let read_result = reader.read_exact(&mut len_buf).await;

            if read_result.is_err() {
                error!("failed to read from socket {:?}", read_result);
                continue;
            }

            let msg_len = u32::from_be_bytes(len_buf) as usize;
            let mut msg_buf = vec![0u8; msg_len];

            if let Err(e) = reader.read_exact(&mut msg_buf).await {
                error!("failed to read from socket: {}", e);
                continue;
            }
            let received_message: Result<Message, _> = serde_json::from_slice(&msg_buf);
            match received_message {
                Ok(message) => match message.message_type {
                    MessageType::Chat => {
                        println!("Received chat message");
                        println!("Message: {}", message.payload);
                    }
                    MessageType::Register => {
                        println!("Received register message");
                    }
                },
                Err(e) => {
                    eprintln!("Failed to parse message: {}", e);
                }
            }
        }
    });

    let write_task = tokio::spawn(async move {
        let mut reader = BufReader::new(io::stdin());
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).await.is_err() {
                error!("Failed to read from stdin");
                continue; // Continue waiting for new input
            }

            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() {
                println!("Sending message: {}", trimmed_line);

                let chat_message = Message {
                    message_type: MessageType::Chat,
                    payload: trimmed_line.to_string(),
                    // other fields
                };

                let encoded_message = chat_message.encode();
                if let Err(e) = writer.write_all(&encoded_message).await {
                    error!("Failed to write to socket: {}", e);
                    line.clear();
                    return;
                }
            }
            line.clear();
        }
    });
    // Optionally, you can await these tasks if you want the main function to wait for them.
    let result = tokio::try_join!(read_task, write_task);
    if let Err(e) = result {
        eprintln!("Failed to join read/write tasks: {}", e);
    }

    Ok(())
}
pub fn send_message(message: String) {
    println!("Sending message: {}", message);
}
