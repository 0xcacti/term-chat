pub mod error;

use axum::{routing::get, Router, Server};
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
use tokio::{
    signal::unix::{signal, SignalKind},
    sync::broadcast,
};

use crate::{api, message::TextMessage, ws};

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
            address: "127.0.0.1:8080".to_string(),
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

pub struct AppState {
    pub user_set: Mutex<HashSet<String>>,
    pub tx: broadcast::Sender<TextMessage>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10);
        let user_set = Mutex::new(HashSet::new());
        Self { user_set, tx }
    }
}

pub async fn run(config: ServerConfig) -> Result<(), error::ServerError> {
    let state = Arc::new(AppState::new());
    // let api_routes = api::routes(Arc::clone(&state)).with_state(Arc::clone(&state));
    let api_routes = api::routes();
    let ws_routes = ws::routes();
    let app = api_routes.nest("/", ws_routes).with_state(state);

    let addr = config.address.parse().unwrap();
    println!("Listening on {}", addr);

    let server = Server::bind(&addr).serve(app.into_make_service());

    let mut sigterm = signal(SignalKind::terminate()).unwrap();
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    tokio::select! {
        _ =  async { server.await.unwrap() } => {
            // Server completed normally
        }
        _ = sigint.recv() => {
            // Received SIGINT signal
            println!("Received SIGINT, shutting down");
        }
        _ = sigterm.recv() => {
            // Received SIGTERM signal
            println!("Received SIGTERM, shutting down");
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::{MessageType, TextMessage};
    use std::sync::atomic::{AtomicU16, Ordering};

    static NEXT_PORT: AtomicU16 = AtomicU16::new(8000);

    fn get_server_address() -> String {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
        format!("127.0.0.1:{}", port)
    }

    fn get_server_config() -> ServerConfig {
        ServerConfig {
            address: get_server_address(),
            ws_enabled: Some(true),
        }
    }

    fn get_test_chat_message() -> TextMessage {
        TextMessage::new(
            MessageType::Text,
            Some("user_one".to_string()),
            "Hello World".to_string(),
        )
    }
    async fn setup(server_address: &str) {
        let config = get_server_config();
    }
}
