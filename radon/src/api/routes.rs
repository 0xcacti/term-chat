use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    Json,
};
use serde_derive::Deserialize;

use crate::config::ServerConfig;

use super::error;

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
}

fn register(
    State(state): State<Arc<ServerConfig>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response<Body>, error::ApiError> {
    todo!()
}
