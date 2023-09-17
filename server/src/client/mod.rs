use tokio::sync::broadcast;
use uuid::Uuid;

pub mod error;

enum ClientState {
    Anonymous,
    Registered { username: String },
}

struct Client {
    id: Uuid,
    state: ClientState,
    addr: std::net::SocketAddr,
    tx: broadcast::Sender<(String, Uuid)>,
}

impl Client {
    pub fn new(addr: std::net::SocketAddr, tx: broadcast::Sender<(String, Uuid)>) -> Self {
        Self {
            id: Uuid::new_v4(),
            state: ClientState::Anonymous,
            addr,
            tx,
        }
    }

    pub fn register(&mut self, username: String) -> Result<(), String> {
        todo!()
    }
}
