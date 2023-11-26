pub mod error;

use clap::Parser;
use figment::providers::Format;
use figment::value::{Dict, Map};
use figment::Provider;
use figment::{error::Error, Figment, Metadata, Profile};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub port: String,
    pub ws_enabled: Option<bool>,
    pub db_user: String,
    pub db_password: String,
    pub db_endpoint: String,
    pub db_port: String,
    pub db_name: String,
    pub db_connection_string: String,
    pub jwt_secret: String,
}

#[derive(Debug, Parser)]
pub struct RunArgs {
    /// The port to run the server on
    #[arg(long = "port", value_name = "port")]
    port: Option<String>,
    // Enable websocket server
    #[arg(long = "websocket", value_name = "ws_enabled")]
    ws: Option<bool>,
    // Database user
    #[arg(short = 'u', long = "db_user", value_name = "db_user")]
    db_user: Option<String>,
    // Database password
    #[arg(short = 'p', long = "db_password", value_name = "db_password")]
    db_password: Option<String>,
    // Database endpoint
    #[arg(short = 'e', long = "db_endpoint", value_name = "db_endpoint")]
    db_endpoint: Option<String>,
    // Database port
    #[arg(long = "db_port", value_name = "db_port")]
    db_port: Option<String>,
    // Database name
    #[arg(long = "db_name", value_name = "db_name")]
    db_name: Option<String>,
    // Database connection string
    #[arg(
        long = "database connection string",
        value_name = "db_connection_string"
    )]
    db_connection_string: Option<String>,
    #[arg(long = "jwt secret")]
    jwt_secret: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: "8080".to_string(),
            ws_enabled: Some(true),
            db_user: "postgres".to_string(),
            db_password: "postgres".to_string(),
            db_endpoint: "localhost".to_string(),
            db_port: "5432".to_string(),
            db_name: "radon".to_string(),
            db_connection_string: "".to_string(),
            jwt_secret: "secret".to_string(),
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
    pub fn compute_db_connection_string(&mut self) {
        self.db_connection_string = format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.db_user, self.db_password, self.db_endpoint, self.db_port, self.db_name
        );
    }
    pub fn merge_with_args(&mut self, args: &RunArgs) {
        if let Some(port) = &args.port {
            self.port = port.clone();
        }
        if let Some(ws_enabled) = &args.ws {
            self.ws_enabled = Some(*ws_enabled);
        }
        if let Some(db_user) = &args.db_user {
            self.db_user = db_user.clone();
        }
        if let Some(db_password) = &args.db_password {
            self.db_password = db_password.clone();
        }
        if let Some(db_endpoint) = &args.db_endpoint {
            self.db_endpoint = db_endpoint.clone();
        }
        if let Some(db_port) = &args.db_port {
            self.db_port = db_port.clone();
        }
        if let Some(db_name) = &args.db_name {
            self.db_name = db_name.clone();
        }
        if let Some(db_connection_string) = &args.db_connection_string {
            self.db_connection_string = db_connection_string.clone();
        }
        if let Some(jwt_secret) = &args.jwt_secret {
            self.jwt_secret = jwt_secret.clone();
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
