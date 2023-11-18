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

pub async fn run(config: ServerConfig) -> Result<(), error::ServerError> {
    let state = Arc::new(AppState::new());
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
            db_connection_string: "postgres://postgres:password@localhost:5432/radon".to_string(),
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
