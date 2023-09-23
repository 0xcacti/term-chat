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
    const SERVER_ADDRESS: &'static str = "127.0.0.1:8080";

    #[tokio::test]
    async fn test_client_can_connect() {
        let mut server = Server::new(SERVER_ADDRESS).await.unwrap();
        let server_handle = tokio::spawn(async move {
            server.run().await.unwrap();
        });

        let client = TcpStream::connect(SERVER_ADDRESS).await;
        assert!(client.is_ok());

        server_handle.abort();
    }

    #[tokio::test]
    async fn test_client_can_send_message() {}

    #[tokio::test]
    async fn test_client_can_receive_message() {}
}
