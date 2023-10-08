use axum::{response::Html, routing::get, Router};
pub mod error;

pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}

pub fn routes() -> Router {
    Router::new().route("/", get(index))
}
