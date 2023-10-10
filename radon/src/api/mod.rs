use std::sync::Arc;

use axum::{
    response::Html,
    routing::{get, post},
    Router,
};

use crate::server::AppState;
pub mod error;

pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}

pub async fn register() {
    todo!()
}

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index))
        .route("/register", post(register))
}
