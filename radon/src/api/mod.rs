use std::{collections::HashMap, convert::Infallible, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    response::Html,
    routing::{get, post},
    Extension, Json, Router,
};
use serde::Deserialize;

use crate::server::AppState;
pub mod error;

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
}

pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/register", post(register))
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response<Body>, Infallible> {
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
