pub mod error;
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

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/ws", get(websocket_handler))
        .with_state(state)
}
async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<ServerConfig>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(state, socket))
}

async fn websocket(state: Arc<ServerConfig>, stream: WebSocket) {}
