pub mod command_handler;
pub mod dispatcher;
pub mod error;

use std::collections::HashMap;

use log::error;
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
};
use uuid::Uuid;

use self::error::ServerError;
use crate::client::Client;

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

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();

            loop {
                tokio::select! {
                    result = reader.read_line(&mut line) => {
                        match result {
                            Ok(n) if n == 0 => break,
                            Ok(_) => {
                                if tx.send((line.clone(), client.id)).is_err() {
                                    break;
                                }
                                line.clear()
                            }
                            Err(e) => {
                                error!("failed to read from socket; err = {:?}", e);
                                break;
                            }

                        }
                    }
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
