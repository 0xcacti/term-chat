use thiserror::Error;

use crate::client::error::ClientError;

#[derive(Error, Debug)]
pub enum ServerError {
    #[error("Failed to bind to address")]
    TcpBind(#[source] std::io::Error),

    #[error("Failed to accept incoming connection")]
    TcpAccept(#[source] std::io::Error),

    #[error("Failed to register client")]
    RegisterClient,
}
