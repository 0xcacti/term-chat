use axum::{
    body::{Body, BoxBody},
    http::{Response, StatusCode},
    response::IntoResponse,
};
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
    fn into_response(self) -> Response<BoxBody> {
        let (status, error_message) = match self {
            UsersError::UsernameTaken => (StatusCode::CONFLICT, "Username already taken"),
            UsersError::Invalid => (StatusCode::CONFLICT, "Invalid username or password"),
            UsersError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            // Handle other error variants...
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = json!({
            "error": error_message
        });

        let body_str = match serde_json::to_string(&body) {
            Ok(b) => b,
            Err(_) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BoxBody::new("Failed to serialize error"))
                    .unwrap()
            }
        };

        Response::builder()
            .status(status)
            .body(BoxBody::new(body_str))
            .unwrap_or_else(|_| {
                Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(BoxBody::new("Failed to build response"))
                    .unwrap()
            })
    }
}
