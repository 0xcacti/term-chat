use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("failed to register username")]
    RegisterUsername,
}
