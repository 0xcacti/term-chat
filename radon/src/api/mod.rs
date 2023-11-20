pub mod error;
pub mod routes;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::Html,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;
use std::{collections::HashMap, convert::Infallible, sync::Arc};

use crate::config::ServerConfig;

pub fn routes() -> Router<Arc<ServerConfig>> {
    Router::new()
}
