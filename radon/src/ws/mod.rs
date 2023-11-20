use std::sync::Arc;

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::IntoResponse,
    routing::get,
    Router,
};
use futures::{SinkExt, StreamExt};

use crate::{
    config::ServerConfig,
    message::{MessageType, TextMessage},
};

pub mod error;

pub fn routes() -> Router<Arc<ServerConfig>> {
    let route_prefix = "/ws";

    Router::new().route(route_prefix, get(websocket_handler))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerConfig>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(state, socket))
}

async fn websocket(state: Arc<ServerConfig>, stream: WebSocket) {}
