pub mod error;

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use axum::routing::get;
use axum::Router;
use log::error;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{io::AsyncWriteExt, net::TcpListener};
use uuid::Uuid;

use crate::message::{Message, MessageType};

struct AppState {
    user_set: Mutex<HashSet<String>>,
    tx: broadcast::Sender<String>,
}

async fn start() -> Result<(), error::ServerError> {
    let user_set = Mutex::new(HashSet::new());
    let (tx, _) = broadcast::channel(100);
    let app_state = Arc::new(AppState { user_set, tx });
    let app = Router::new()
        .route("/", get(index))
        .route("/ws", get(websocket_handler))
        .with_state(app_state);
    let listener = TcpListener::bind("127.0.0.1:8080").await.map_err(|e| {
        error!("failed to bind to socket; err = {:?}", e);
        error::ServerError::TcpBind(e)
    })?;
    axum::serve(listener, app).await.unwrap();

    Ok(())
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
