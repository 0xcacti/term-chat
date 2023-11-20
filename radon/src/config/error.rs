use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Invalid configuration")]
    Invalid(#[source] figment::Error),
}
