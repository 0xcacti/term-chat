use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AuthError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Invalid request")]
    Invalid,
}

// should I implement into response or just respond_with_json
impl IntoResponse for AuthError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AuthError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            AuthError::Invalid => (StatusCode::BAD_REQUEST, "Invalid request".to_string()),
            // handle other variants
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}
