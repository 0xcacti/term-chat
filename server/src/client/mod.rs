pub mod error;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Failed to bind to address")]
    TcpBind(#[source] std::io::Error),

    #[error("Failed to accept incoming connection")]
    TcpAccept(#[source] std::io::Error),

    #[error("Failed to register client")]
    RegisterClient,
}

use self::error::ClientError;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Clone)]
pub enum ClientState {
    Anonymous,
    Registered { username: String }, // TODO: make sure this isn't too long so copying isn't
                                     // expensive
}

#[derive(Clone)]
pub struct Client {
    pub id: Uuid,
    pub state: ClientState,
    pub addr: std::net::SocketAddr,
    pub tx: broadcast::Sender<(Vec<u8>, Uuid)>,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, tx: broadcast::Sender<(Vec<u8>, Uuid)>) -> Self {
        Self {
            id: Uuid::new_v4(),
            state: ClientState::Anonymous,
            addr,
            tx,
        }
    }

    pub fn register(&mut self, username: String) -> Result<(), ClientError> {
        // is registered
        self.state = ClientState::Registered { username };
        Ok(())
    }
}
