use axum::response::Html;

pub mod error;
pub async fn index() -> Html<&'static str> {
    Html(std::include_str!("../chat.html"))
}
