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

use crate::message::{MessageType, TextMessage};

pub mod error;

pub fn routes() -> Router<Arc<AppState>> {
    let route_prefix = "/ws";

    Router::new().route(route_prefix, get(websocket_handler))
}

async fn websocket_handler(
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| websocket(state, socket))
}

async fn websocket(state: Arc<AppState>, stream: WebSocket) {
    let (mut sender, mut receiver) = stream.split();
    let username = String::new();
    // check that user exists

    let mut rx = state.tx.subscribe();
    let msg = TextMessage::new(
        MessageType::Join,
        Some(username.clone()),
        format!("{} joined.", username).to_string(),
    );

    let _ = state.tx.send(msg);

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            let serialized = serde_json::to_string(&msg).unwrap();
            if sender.send(Message::Text(serialized)).await.is_err() {
                break;
            }
        }
    });
    let tx = state.tx.clone();
    let name = username.clone();
    let mut receive_task = tokio::spawn(async move {
        while let Some(Ok(Message::Text(msg))) = receiver.next().await {
            let msg: TextMessage = serde_json::from_str(&msg).unwrap();
            let _ = tx.send(msg);
        }
    });

    tokio::select! {
        _ = (&mut send_task) => receive_task.abort(),
        _ = (&mut receive_task) => send_task.abort(),
    }

    let msg = TextMessage::new(
        MessageType::Leave,
        Some(username.clone()),
        format!("{} left the chat.", username).to_string(),
    );
    let _ = state.tx.send(msg);
    state.user_set.lock().unwrap().remove(&username);
}
