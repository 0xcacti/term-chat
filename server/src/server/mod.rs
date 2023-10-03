pub mod error;

use std::collections::HashMap;

use log::error;
use tokio::io::AsyncReadExt;
use tokio::net::tcp::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{self, Receiver, Sender};
use tokio::{io::AsyncWriteExt, net::TcpListener};
use uuid::Uuid;

use self::error::ServerError;
use crate::client::Client;
use crate::message::{Message, MessageType};

pub struct Server {
    listener: TcpListener,
    clients: HashMap<Uuid, Client>,
    broadcast_tx: broadcast::Sender<(Vec<u8>, Uuid)>,
}

impl Server {
    pub async fn new(addr: &str) -> Result<Self, ServerError> {
        let listener = TcpListener::bind(addr)
            .await
            .map_err(ServerError::TcpBind)?;
        let clients = HashMap::new();
        let (broadcast_tx, _) = broadcast::channel(100);

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
        println!("meow");

        tokio::spawn(async move {
            let (mut reader, mut writer) = socket.split();
            loop {
                tokio::select! {
                    read_result = read_from_cilent(tx.clone(), &mut reader, &client) => {
                        if let Err(e) = read_result {
                            error!("failed to read from client: {}", e);
                            break;
                        }
                    }

                    write_result = send_to_client(&mut rx, &mut writer, &client) => {
                        if let Err(e) = write_result {
                            error!("failed to write to client: {}", e);
                            break;
                        }
                    }

                }
            }
        });

        Ok(())
    }
}
async fn send_to_client(
    rx: &mut Receiver<(Vec<u8>, Uuid)>,
    writer: &mut WriteHalf<'_>,
    client: &Client,
) -> Result<(), ServerError> {
    let result = rx.recv().await;
    match result {
        Ok((msg, other_id)) => {
            if client.id != other_id {
                println!("everything is gonna be okay#");
                if writer.write_all(&msg).await.is_err() {
                    error!("failed to write to socket");
                }
            } else {
                println!("everything is gonna be okay");
            }
        }
        Err(broadcast::error::RecvError::Lagged(_)) => {
            error!("lagged");
        }
        Err(broadcast::error::RecvError::Closed) => {
            error!("channel closed");
        }
    }
    Ok(())
}

async fn read_from_cilent(
    tx: Sender<(Vec<u8>, Uuid)>,
    reader: &mut ReadHalf<'_>,
    client: &Client,
) -> Result<(), ServerError> {
    let mut len_buf = [0u8; 4];
    reader.read_exact(&mut len_buf).await.unwrap(); // TODO: handle error
    let msg_len = u32::from_be_bytes(len_buf) as usize;
    let mut msg_buf = vec![0u8; msg_len as usize];
    if let Err(e) = reader.read_exact(&mut msg_buf).await {
        error!("failed to read from socket {e}");
    }

    let message: Result<Message, _> = serde_json::from_slice(&msg_buf);
    match message {
        Ok(parsed_message) => match parsed_message.message_type {
            MessageType::Chat => {
                println!("chat message");
                tx.send((parsed_message.encode(), client.id)).unwrap();
            }
            MessageType::Register => {
                println!("register message");
            }
        },
        Err(e) => {
            error!("failed to parse message: {}", e);
        }
    }
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
        let message_json = serde_json::to_string(&message).unwrap();
        let message_len = message_json.len() as u32;
        let message_len_buf = message_len.to_be_bytes().to_vec();

        let mut message_buf = message_len_buf;
        message_buf.extend_from_slice(message_json.as_bytes());
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
        client_one.write_all(&message_buf.clone()).await.unwrap();

        let mut len_buf = [0u8; 4];
        client_two.read_exact(&mut len_buf).await.unwrap();
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut msg_buf = vec![0u8; msg_len];
        client_two.read_exact(&mut msg_buf).await.unwrap();

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

        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("meow -- we are sending the next message");
        let message_buf = get_test_chat_message();
        client_one.write_all(&message_buf.clone()).await.unwrap();
        println!("meow -- we successfully wrote the next message");

        let mut len_buf = [0u8; 4];
        client_two.read_exact(&mut len_buf).await.unwrap();
        let msg_len = u32::from_be_bytes(len_buf) as usize;
        println!("meow -- we read the message length");

        let mut msg_buf = vec![0u8; msg_len];
        client_two.read_exact(&mut msg_buf).await.unwrap();

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
        let message_buf = get_test_chat_message();
        client_one.write_all(&message_buf.clone()).await.unwrap();

        let mut len_buf = [0u8; 4];
        client_two.read_exact(&mut len_buf).await.unwrap();
        let msg_len = u32::from_be_bytes(len_buf) as usize;

        let mut msg_buf = vec![0u8; msg_len];
        client_two.read_exact(&mut msg_buf).await.unwrap();

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
