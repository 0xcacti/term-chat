pub mod error;

use std::collections::HashMap;

use log::error;
use tokio::io::AsyncReadExt;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::{io::AsyncWriteExt, net::TcpListener};
use uuid::Uuid;

use self::error::ServerError;
use crate::client::Client;
use crate::message::Message;

pub struct Server {
    listener: TcpListener,
    clients: HashMap<Uuid, Client>,
    broadcast_tx: broadcast::Sender<(String, Uuid)>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(ServerError::TcpBind)?;
        let clients = HashMap::new();
        let (broadcast_tx, _) = broadcast::channel(10);

        Ok(Self {
            listener,
            clients,
            broadcast_tx,
        })
    }

    pub async fn register_client(
        &mut self,
        addr: std::net::SocketAddr,
    ) -> Result<Client, ServerError> {
        let client = Client::new(addr, self.broadcast_tx.clone());
        self.clients.insert(client.id, client.clone());
        Ok(client)
    }

    pub async fn run(&mut self) -> Result<(), ServerError> {
        loop {
            let (socket, addr) = self
                .listener
                .accept()
                .await
                .map_err(ServerError::TcpAccept)?;
            let client = self.register_client(addr).await?;
            self.handle_client(client, socket).await?;
        }
    }

    pub async fn handle_client(
        &mut self,
        client: Client,
        mut socket: TcpStream,
    ) -> Result<(), ServerError> {
        let tx = self.broadcast_tx.clone();
        let mut rx = tx.subscribe();
        let mut len_buf = [0u8; 4];

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            loop {
                tokio::select! {
                    // read from the client
                    read_result = reader.read_exact(&mut len_buf) => {
                        if read_result.is_err() {
                            error!("failed to read from socket");
                            break;
                        }
                        let msg_len = u32::from_be_bytes(len_buf) as usize;

                        let mut msg_buf = vec![0u8; msg_len as usize];

                        if reader.read_exact(&mut msg_buf).await.is_err() {
                            error!("failed to read from socket");
                            break;
                        }

                        let message: Result<Message, _> = serde_json::from_slice(&msg_buf);
                        match message {
                            Ok(parsed_message) => {

                                println!("message: {:?}", parsed_message);

                            }
                            Err(e) => {
                                error!("failed to parse message: {}", e);
                                break;
                            }
                        }
                    }

                    // read from other clients and broadcast

                    result = rx.recv() => {
                        match result {
                            Ok((msg, other_id)) if client.id != other_id => {
                                if writer.write_all(msg.as_bytes()).await.is_err() {
                                    error!("failed to write to socket");
                                    break;
                                }

                            }
                            Err(broadcast::error::RecvError::Lagged(_)) => {
                                error!("lagged");
                                break;
                            }
                            Err(broadcast::error::RecvError::Closed) => {
                                error!("channel closed");
                                break;
                            }

                            _ => (),

                        }
                    }
                }
            }
        });
        Ok(())
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
        let message_json = serde_json::to_string(&message).unwrap();
        let message_len = message_json.len() as u32;
        let mut message_len_buf = message_len.to_be_bytes().to_vec();
        message_len_buf.resize(4, 0);

        let mut message_buf = message_json.as_bytes().to_vec();
        message_buf.extend_from_slice(&mut message_len_buf);
        message_buf
    }
    async fn setup(server_address: &str) -> JoinHandle<()> {
        let mut server = Server::new(server_address).await.unwrap();
        let server_handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });
        server_handle
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
        client_one.write_all(&message_buf).await.unwrap();

        let mut len_buf = [0u8; 4];
        client_two.read_exact(&mut len_buf).await.unwrap();
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let msg_buf = vec![0u8; msg_len];
        client_two.read_exact(&mut len_buf).await.unwrap();

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

        server_handle.abort();
    }
}
