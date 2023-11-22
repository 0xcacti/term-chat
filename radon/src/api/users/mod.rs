pub mod error;

use std::sync::Arc;

use super::error;
use crate::config::ServerConfig;
use axum::{
    body::Body,
    extract::State,
    http::{Response, StatusCode},
    routing::post,
    Extension, Json, Router,
};
use serde_derive::Deserialize;

pub fn router() -> Router {
    Router::new().route("/users", post(create_user))
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
    password: String,
}

async fn create_user(
    db: Extension<PgPool>,
    Json(req): Json<RegisterRequest>,
) -> Result<StatusCode> {
    req.validate()?;

    let RegisterRequest { username, password } = req;

    // It would be irresponsible to store passwords in plaintext, however.
    let password_hash = crate::password::hash(password).await?;

    sqlx::query!(
        // language=PostgreSQL
        r#"
            insert into "user"(username, password_hash)
            values ($1, $2)
        "#,
        username,
        password_hash
    )
    .execute(&*db)
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(dbe) if dbe.constraint() == Some("user_username_key") => {
            Error::Conflict("username taken".into())
        }
        _ => e.into(),
    })?;

    Ok(StatusCode::NO_CONTENT)
}

impl RegisterRequest {
    // NOTE: normally we wouldn't want to verify the username and password every time,
    // but persistent sessions would have complicated the example.
    pub async fn verify(self, db: impl PgExecutor<'_> + Send) -> Result<UserId> {
        self.validate()?;

        let maybe_user = sqlx::query!(
            r#"select user_id, password_hash from "user" where username = $1"#,
            self.username
        )
        .fetch_optional(db)
        .await?;

        if let Some(user) = maybe_user {
            let verified = crate::password::verify(self.password, user.password_hash).await?;

            if verified {
                return Ok(user.user_id);
            }
        }

        // Sleep a random amount of time to avoid leaking existence of a user in timing.
        let sleep_duration =
            rand::thread_rng().gen_range(Duration::from_millis(100)..=Duration::from_millis(500));
        tokio::time::sleep(sleep_duration).await;

        Err(Error::UnprocessableEntity(
            "invalid username/password".into(),
        ))
    }
}
