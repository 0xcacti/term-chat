pub mod error;

use std::{sync::Arc, time::Duration};

use crate::config::ServerConfig;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    routing::post,
    Extension, Json, Router,
};
use error::UsersError;
use rand::Rng;
use serde_derive::Deserialize;
use sqlx::{PgExecutor, PgPool};
use uuid::Uuid;
use validator::Validate;

use super::AppState;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/users", post(create_user))
        .with_state(state)
}

#[derive(Deserialize, Validate)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterResponse {
    user_id: String,
    username: String,
}

#[axum_macros::debug_handler]
async fn create_user(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RegisterRequest>,
) -> Response<Body> {
    // println!("req: {:?}", "hello");
    req.validate().map_err(|_| UsersError::Invalid)?;

    let RegisterRequest { username, password } = req;

    // It would be irresponsible to store passwords in plaintext, however.
    //let password_hash = crate::password::hash(password).await?;
    let password_hash = "password_hash".to_string();
    let time = chrono::Utc::now().naive_utc();

    let res = sqlx::query!(
        // language=PostgreSQL
        r#"
            insert into "users"(username, password_hash, created_at, updated_at)
            values ($1, $2, $3, $4)
        "#,
        username,
        password_hash,
        time.clone(),
        time
    )
    .execute(&state.db)
    .await;
    match res {
        Ok(_) => {
            return Ok((
                StatusCode::CREATED,
                Json(RegisterResponse {
                    user_id: "user_id".to_string(),
                    username,
                }),
            ))
        }
        Err(sqlx::Error::Database(dbe)) if dbe.constraint() == Some("user_username_key") => {
            return Err(UsersError::UsernameTaken)
        }
        Err(e) => return Err(UsersError::Database(e)),
    };
}

impl RegisterRequest {
    // NOTE: normally we wouldn't want to verify the username and password every time,
    // but persistent sessions would have complicated the example.
    pub async fn verify(self, db: impl PgExecutor<'_> + Send) -> Result<Uuid, UsersError> {
        self.validate().map_err(|_| UsersError::Invalid)?;

        let maybe_user = sqlx::query!(
            r#"select user_id, password_hash from "users" where username = $1"#,
            self.username
        )
        .fetch_optional(db)
        .await?;

        if let Some(user) = maybe_user {
            // let verified = crate::password::verify(self.password, user.password_hash).await?;
            let verified = true;

            if verified {
                return Ok(user.user_id);
            }
        }

        // Sleep a random amount of time to avoid leaking existence of a user in timing.
        let sleep_duration =
            rand::thread_rng().gen_range(Duration::from_millis(100)..=Duration::from_millis(500));
        tokio::time::sleep(sleep_duration).await;

        Err(UsersError::Invalid)
    }
}
