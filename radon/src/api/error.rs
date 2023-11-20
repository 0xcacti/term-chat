use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Username already taken")]
    UsernameTaken,
    #[error("User not found")]
    UserNotFound,
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Invalid token")]
    InvalidToken,
    #[error("Internal error")]
    InternalError,
}
