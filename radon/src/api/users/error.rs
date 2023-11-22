use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsersError {
    #[error("Invalid configuration")]
    Invalid(#[source] figment::Error),
}
