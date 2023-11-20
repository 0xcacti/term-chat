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
    let mut user_set = state.user_set.lock().unwrap();

    if user_set.contains(&payload.username) {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Username already taken".into())
            .unwrap();
        return Ok(response);
    } else {
        user_set.insert(payload.username);
        let response = Response::builder()
            .status(StatusCode::CREATED)
            .body("User registered".into())
            .unwrap();
        return Ok(response);
    }
}
