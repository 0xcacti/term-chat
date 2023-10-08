pub mod error;

use axum::{routing::get, Router};
use clap::Parser;
use figment::{
    providers::Format,
    value::{Dict, Map},
    Error, Figment, Metadata, Profile, Provider,
};
use serde::Serialize;
use serde_derive::Deserialize;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

use crate::{api, ws};

#[derive(Debug, Parser)]
pub struct RunArgs {
    /// The address to run the server on including port
    #[arg(long = "address", value_name = "address:port")]
    address: Option<String>,
    // Enable websocket server
    #[arg(long = "websocket", value_name = "ws_enabled")]
    ws: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
    pub ws_enabled: Option<bool>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080 -- my default".to_string(),
            ws_enabled: Some(true),
        }
    }
}

impl ServerConfig {
    pub fn merge_with_args(&mut self, args: &RunArgs) {
        if let Some(address) = &args.address {
            self.address = address.clone();
        }
        if let Some(ws_enabled) = &args.ws {
            self.ws_enabled = Some(*ws_enabled);
        }
    }

    pub fn from<T: Provider>(provider: T) -> Result<ServerConfig, error::ServerError> {
        Figment::from(provider)
            .extract()
            .map_err(error::ServerError::ConfigError)
    }

    pub fn figment() -> Figment {
        use figment::providers::{Env, Toml};
        Figment::from(Self::default())
            .merge(Toml::file("radon.toml"))
            .merge(Env::prefixed("RADON_"))
    }
}

impl Provider for ServerConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Radon Server Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(ServerConfig::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}

pub struct Server {
    config: ServerConfig,
    router: Router,
    state: Arc<AppState>,
}

pub struct AppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<String>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10);
        let user_set = Mutex::new(HashSet::new());
        Self { user_set, tx }
    }
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        let state = Arc::new(AppState::new());
        let router = Router::new();
        Self {
            config,
            router,
            state,
        }
    }

    pub async fn run(&mut self) -> Result<(), error::ServerError> {
        // self.router = self.router.route("/", get(api::index));

        // steps -
        // start rest API
        // if enabled start ws api
        // listen for shutdown signal and shutdown gracefully
        if let Some(ws_enabled) = self.config.ws_enabled {
            if ws_enabled {
                self.router
                    .nest("/", api::routes())
                    .nest("/ws", ws::routes(self.state.clone()));
            } else {
                self.router.nest("/", api::routes());
            }
        }

        Ok(())
    }
}

// #[cfg(test)]
// mod test {
//     use super::*;
//     use crate::message::MessageType;
//     use std::sync::atomic::{AtomicU16, Ordering};
//     use tokio::task::JoinHandle;
//
//     static NEXT_PORT: AtomicU16 = AtomicU16::new(8000);
//
//     fn get_server_address() -> String {
//         let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
//         format!("127.0.0.1:{}", port)
//     }
//
//     fn get_test_chat_message() -> Vec<u8> {
//         let message = Message::new(MessageType::Chat, "hello".to_string());
//         message.encode()
//     }
//     async fn setup(server_address: &str) -> JoinHandle<()> {
//         let mut server = Server::new(server_address).await.unwrap();
//         let server_handle = tokio::spawn(async move {
//             server.run().await.unwrap();
//         });
//         server_handle
//     }
//
//     async fn read_and_validate_message(client: &mut TcpStream) {
//         let mut len_buf = [0u8; 4];
//         client.read_exact(&mut len_buf).await.unwrap();
//         let msg_len = u32::from_be_bytes(len_buf) as usize;
//
//         let mut msg_buf = vec![0u8; msg_len];
//         client.read_exact(&mut msg_buf).await.unwrap();
//         let received_message: Result<Message, _> = serde_json::from_slice(&msg_buf);
//         match received_message {
//             Ok(message) => {
//                 assert_eq!(message.message_type, MessageType::Chat);
//                 assert_eq!(message.payload, "hello");
//             }
//             Err(e) => {
//                 panic!("failed to parse message: {}", e);
//             }
//         }
//     }
//
//     #[tokio::test]
//     async fn test_client_can_connect() {
//         let server_address = get_server_address();
//         let server_handle = setup(&server_address).await;
//
//         let client = TcpStream::connect(server_address).await;
//         assert!(client.is_ok());
//
//         server_handle.abort();
//     }
//
//     #[tokio::test]
//     async fn test_client_can_send_message() {
//         let server_address = get_server_address();
//         let server_handle = setup(&server_address).await;
//         let mut client = TcpStream::connect(server_address).await.unwrap();
//         let message_buf = get_test_chat_message();
//         client.write_all(&message_buf).await.unwrap();
//         server_handle.abort();
//     }
//
//     #[tokio::test]
//     async fn test_client_can_receive_message() {
//         let server_address = get_server_address();
//         let server_handle = setup(&server_address).await;
//         let mut client_one = TcpStream::connect(&server_address).await.unwrap();
//         let mut client_two = TcpStream::connect(&server_address).await.unwrap();
//         let message_buf = get_test_chat_message();
//         client_one.write_all(&message_buf.clone()).await.unwrap();
//         read_and_validate_message(&mut client_two).await;
//         client_one.write_all(&message_buf.clone()).await.unwrap();
//         read_and_validate_message(&mut client_two).await;
//         client_one.write_all(&message_buf.clone()).await.unwrap();
//         read_and_validate_message(&mut client_two).await;
//         server_handle.abort();
//     }
// }
