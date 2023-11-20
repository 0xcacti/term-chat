pub mod error;

use clap::Parser;
use figment::providers::Format;
use figment::value::{Dict, Map};
use figment::Provider;
use figment::{error::Error, Figment, Metadata, Profile};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    pub address: String,
    pub ws_enabled: Option<bool>,
    pub db_connection_string: String,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    /// The address to run the server on including port
    #[arg(long = "address", value_name = "address:port")]
    address: Option<String>,
    // Enable websocket server
    #[arg(long = "websocket", value_name = "ws_enabled")]
    ws: Option<bool>,
    // Database connection string
    #[arg(
        long = "database connection string",
        value_name = "db_connection_string"
    )]
    db_connection_string: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            address: "127.0.0.1:8080".to_string(),
            ws_enabled: Some(true),
            db_connection_string: "postgres://postgres:password@localhost:5432/radon".to_string(),
        }
    }
}

impl Provider for ServerConfig {
    fn metadata(&self) -> Metadata {
        Metadata::named("Radon Server Config")
    }

    fn data(&self) -> Result<Map<Profile, Dict>, Error> {
        figment::providers::Serialized::defaults(ServerConfig::default()).data()
    }

    fn profile(&self) -> Option<Profile> {
        None
    }
}

impl ServerConfig {
    pub fn merge_with_args(&mut self, args: &RunArgs) {
        if let Some(address) = &args.address {
            self.address = address.clone();
        }
        if let Some(ws_enabled) = &args.ws {
            self.ws_enabled = Some(*ws_enabled);
        }
    }

    pub fn from<T: Provider>(provider: T) -> Result<ServerConfig, error::ConfigError> {
        Figment::from(provider)
            .extract()
            .map_err(error::ConfigError::Invalid)
    }

    pub fn figment() -> Figment {
        use figment::providers::{Env, Toml};
        Figment::from(Self::default())
            .merge(Toml::file("radon.toml"))
            .merge(Env::prefixed("RADON_"))
    }
}
