use axum::{http::StatusCode, response::IntoResponse, Json};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UsersError {
    #[error("Invalid username or password")]
    Invalid,
    #[error("Username is taken")]
    UsernameTaken,
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

// should I implement into response or just respond_with_json
impl IntoResponse for UsersError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            UsersError::Invalid => (StatusCode::BAD_REQUEST, "Invalid request".to_string()),
            UsersError::UsernameTaken => (StatusCode::CONFLICT, "Username is taken".to_string()),
            UsersError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            ),
            // handle other variants
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}
