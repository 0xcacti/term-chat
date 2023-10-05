pub mod error;

use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use futures::{sink::SinkExt, stream::StreamExt};
use log::error;
use std::{
    collections::HashSet,
    sync::{Arc, Mutex},
};
use tokio::sync::broadcast;

struct Server {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

impl Server {
    pub async fn start() -> Result<(), error::ServerError> {
        let user_set = Mutex::new(HashSet::new());
        let (tx, _) = broadcast::channel(100);
        let app_state = Arc::new(AppState { user_set, tx });
        let app = Router::new()
            .route("/", get(index))
            .route("/ws", get(websocket_handler))
            .with_state(app_state);
        axum::Server::bind(&"127.0.0.1:8080".parse().unwrap())
            .serve(app.into_make_service())
            .await
            .unwrap();

        Ok(())
    }

    async fn websocket_handler(
        ws: WebSocketUpgrade,
        State(app_state): State<Arc<AppState>>,
    ) -> impl IntoResponse {
        ws.on_upgrade(|socket| websocket(socket, app_state))
    }

    async fn websocket(stream: WebSocket, state: Arc<AppState>) {
        let (mut sender, mut receiver) = stream.split();
        let mut username = String::new();
        while let Some(Ok(message)) = receiver.next().await {
            if let Message::Text(name) = message {
                check_username(&state, &mut username, &name);
                if !username.is_empty() {
                    break;
                } else {
                    let _ = sender
                        .send(Message::Text("Username already taken".to_string()))
                        .await;
                }
                return;
            }
        }
        let mut rx = state.tx.subscribe();
        let msg = format!("{username} joined.");
        println!("{}", msg);
        let _ = state.tx.send(msg);

        let mut send_task = tokio::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });
        let tx = state.tx.clone();
        let name = username.clone();
        let mut receive_task = tokio::spawn(async move {
            while let Some(Ok(Message::Text(msg))) = receiver.next().await {
                let _ = tx.send(format!("{name}: {msg}"));
            }
        });

        tokio::select! {
            _ = (&mut send_task) => receive_task.abort(),
            _ = (&mut receive_task) => send_task.abort(),
        }

        let msg = format!("{username} left.");
        println!("{}", msg);
        let _ = state.tx.send(msg);
        state.user_set.lock().unwrap().remove(&username);
    }

    async fn index() -> Html<&'static str> {
        Html(std::include_str!("../chat.html"))
    }

    fn check_username(state: &AppState, username: &mut String, name: &str) {
        let mut user_set = state.user_set.lock().unwrap();
        if !user_set.contains(name) {
            user_set.insert(name.to_string());
            username.push_str(name);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::message::MessageType;
    use std::sync::atomic::{AtomicU16, Ordering};
    use tokio::task::JoinHandle;

    static NEXT_PORT: AtomicU16 = AtomicU16::new(8000);

    fn get_server_address() -> String {
        let port = NEXT_PORT.fetch_add(1, Ordering::SeqCst);
        format!("127.0.0.1:{}", port)
    }

    fn get_test_chat_message() -> Vec<u8> {
        let message = Message::new(MessageType::Chat, "hello".to_string());
        message.encode()
    }
    async fn setup(server_address: &str) -> JoinHandle<()> {
        let mut server = Server::new(server_address).await.unwrap();
        let server_handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });
        server_handle
    }

    async fn read_and_validate_message(client: &mut TcpStream) {
        let mut len_buf = [0u8; 4];
        client.read_exact(&mut len_buf).await.unwrap();
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut msg_buf = vec![0u8; msg_len];
        client.read_exact(&mut msg_buf).await.unwrap();
        let received_message: Result<Message, _> = serde_json::from_slice(&msg_buf);
        match received_message {
            Ok(message) => {
                assert_eq!(message.message_type, MessageType::Chat);
                assert_eq!(message.payload, "hello");
            }
            Err(e) => {
                panic!("failed to parse message: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_client_can_connect() {
        let server_address = get_server_address();
        let server_handle = setup(&server_address).await;

        let client = TcpStream::connect(server_address).await;
        assert!(client.is_ok());

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_client_can_send_message() {
        let server_address = get_server_address();
        let server_handle = setup(&server_address).await;
        let mut client = TcpStream::connect(server_address).await.unwrap();
        let message_buf = get_test_chat_message();
        client.write_all(&message_buf).await.unwrap();
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_client_can_receive_message() {
        let server_address = get_server_address();
        let server_handle = setup(&server_address).await;
        let mut client_one = TcpStream::connect(&server_address).await.unwrap();
        let mut client_two = TcpStream::connect(&server_address).await.unwrap();
        let message_buf = get_test_chat_message();
        client_one.write_all(&message_buf.clone()).await.unwrap();
        read_and_validate_message(&mut client_two).await;
        client_one.write_all(&message_buf.clone()).await.unwrap();
        read_and_validate_message(&mut client_two).await;
        client_one.write_all(&message_buf.clone()).await.unwrap();
        read_and_validate_message(&mut client_two).await;
        server_handle.abort();
    }
}
