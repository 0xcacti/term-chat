use thiserror::Error;

#[derive(Error, Debug)]
pub enum DBError {
    #[error("Failed to open connection to database")]
    Connection(#[source] rusqlite::Error),
    #[error("Failed to setup database")]
    Setup(#[source] rusqlite::Error),
    #[error("Failed to execute query")]
    Query(#[source] rusqlite::Error),
    #[error("User already exists")]
    UserExists,
}
