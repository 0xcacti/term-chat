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

pub mod error;

pub struct AppState {
    // pub db: Arc<sqlx::Pool<sqlx::Postgres>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(10);
        let user_set = Mutex::new(HashSet::new());
        Self {}
    }
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/register", post(register))
}
