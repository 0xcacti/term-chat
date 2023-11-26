pub mod error;
pub mod utils;

use rand::Rng;
use std::{sync::Arc, time::Duration};
use uuid::Uuid;

use axum::{
    body::{Body, Bytes},
    http::{Response, StatusCode},
    Extension, Json,
};
use serde_derive::{Deserialize, Serialize};
use sqlx::PgExecutor;
use validator::Validate;

use self::error::AuthError;

use super::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
    pub expires_in_seconds: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    pub email: String,
    pub id: usize,
    pub token: String,
    pub refresh_token: String,
}

pub async fn login(
    Extension(state): Extension<Arc<AppState>>,
    Json(login_attempt): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), AuthError> {
    let user_id = login_attempt.verify(&state.db).await?;
    match user {
        Ok(user) => {
            if !bcrypt::verify(&user_candidate.password, &user.password).unwrap() {
                return respond_with_error(StatusCode::UNAUTHORIZED, "something went wrong");
            }

            let access_token_expiry = 60 * 60; // 1 hour
            let refresh_token_expiry = 60 * 60 * 24 * 60; // 60 days

            let access_jwt = make_jwt(
                user.id,
                "chirpy-access".to_string(),
                &state.lock().await.jwt_secret,
                Duration::from_secs(access_token_expiry),
            )
            .unwrap();

            let refresh_jwt = make_jwt(
                user.id,
                "chirpy-refresh".to_string(),
                &state.lock().await.jwt_secret,
                Duration::from_secs(refresh_token_expiry),
            )
            .unwrap();

            return respond_with_json(
                serde_json::to_string(&user.into_login_response(access_jwt, refresh_jwt)).unwrap(),
                StatusCode::OK,
            );
        }
        Err(DBError::UserNotFound) => {
            return respond_with_error(StatusCode::NOT_FOUND, "User not found")
        }
        Err(_) => {
            return respond_with_error(StatusCode::INTERNAL_SERVER_ERROR, "something went wrong")
        }
    }
}

impl LoginRequest {
    // NOTE: normally we wouldn't want to verify the username and password every time,
    // but persistent sessions would have complicated the example.
    pub async fn verify(self, db: impl PgExecutor<'_> + Send) -> Result<Uuid, AuthError> {
        let maybe_user = sqlx::query!(
            r#"select user_id, password_hash from "users" where username = $1"#,
            self.username
        )
        .fetch_optional(db)
        .await?;

        if let Some(user) = maybe_user {
            let verified = crate::api::auth::utils::verify(self.password, user.password_hash)
                .await
                .map_err(|_| AuthError::Invalid)?;

            if verified {
                return Ok(user.user_id);
            }
        }

        // Sleep a random amount of time to avoid leaking existence of a user in timing.
        let random_millis = rand::thread_rng().gen_range(100..=500);
        let sleep_duration = Duration::from_millis(random_millis);
        tokio::time::sleep(sleep_duration).await;

        Err(AuthError::Invalid)
    }
}
