pub mod message;

use anyhow::Result;
use log::error;
use message::{Message, MessageType};
use serde::Deserialize;
use tokio::{
    io::{
        self, stdin, stdout, AsyncBufRead, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader,
    },
    net::TcpStream,
    task,
};

use futures::{sink::SinkExt, stream::StreamExt};
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

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
            stdout().flush().await.unwrap();
            tokio::task::yield_now().await;
        }
    });

    let write_task = tokio::spawn(async move {
        let mut reader = BufReader::new(stdin());
        loop {
            let mut input_buffer = Vec::new();
            // Wait for input from the terminal
            println!("Enter text to send: ");
            stdout().flush().await.unwrap();

            if reader.read_until(b'\n', &mut input_buffer).await.is_err() {
                eprintln!("Failed to read from stdin");
                break;
            }

            let message_str = match String::from_utf8(input_buffer.clone()) {
                Ok(v) => v,
                Err(e) => {
                    eprintln!("Invalid UTF-8 sequence: {}", e);
                    continue;
                }
            };

            let message = Message::new(MessageType::Chat, message_str);
            let serialized_message = message.encode();

            // Write the input to the writer
            if let Err(e) = writer.write_all(&serialized_message).await {
                eprintln!("Failed to write to socket: {}", e);
                break;
            }

            tokio::task::yield_now().await;
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
