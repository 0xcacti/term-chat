use std::time::Duration;

use anyhow::{Context, Result};
use chrono::Utc;
use jsonwebtoken::{
    decode, encode, Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation,
};
use serde_derive::{Deserialize, Serialize};
use tokio::task;

use argon2::password_hash::SaltString;
use argon2::{password_hash, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    iss: String,
    iat: i64,
    exp: i64,
    sub: String,
}

pub async fn hash(password: String) -> Result<String> {
    task::spawn_blocking(move || {
        let salt = SaltString::generate(rand::thread_rng());
        Ok(Argon2::default()
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!(e).context("failed to hash password"))?
            .to_string())
    })
    .await
    .context("panic in hash()")?
}

pub async fn verify(password: String, hash: String) -> anyhow::Result<bool> {
    task::spawn_blocking(move || {
        let hash = PasswordHash::new(&hash)
            .map_err(|e| anyhow::anyhow!(e).context("BUG: password hash invalid"))?;

        let res = Argon2::default().verify_password(password.as_bytes(), &hash);

        match res {
            Ok(()) => Ok(true),
            Err(password_hash::Error::Password) => Ok(false),
            Err(e) => Err(anyhow::anyhow!(e).context("failed to verify password")),
        }
    })
    .await
    .context("panic in verify()")?
}

pub fn make_jwt(
    user_id: Uuid,
    issuer: String,
    user_secret: &str,
    expires_in: Duration,
) -> Result<String> {
    let now = Utc::now();

    let expiration = now + chrono::Duration::from_std(expires_in)?;
    let claims = Claims {
        iss: issuer,
        iat: now.timestamp(),
        exp: expiration.timestamp(),
        sub: user_id.to_string(),
    };
    let header = Header::new(jsonwebtoken::Algorithm::HS256);
    let encoding_key = EncodingKey::from_secret(user_secret.as_ref());
    let token = encode(&header, &claims, &encoding_key).unwrap();
    Ok(token)
}

pub fn validate_jwt(token: &str, secret: &str) -> Result<(String, String)> {
    // (user_id, issuer)
    let mut validation = Validation::default();
    validation.algorithms = vec![Algorithm::HS256];
    validation.leeway = 0;

    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let token_data: TokenData<Claims> = decode(token, &decoding_key, &validation)?;
    println!("{:?}", token_data);
    let issuer = token_data.claims.iss;
    let user_id = token_data.claims.sub;

    Ok((issuer, user_id))
}
