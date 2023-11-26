pub mod error;
pub mod utils;

use std::sync::Arc;

use axum::{body::Body, http::Response, Extension};

use super::AppState;

pub async fn login(Extension(state): Extension<Arc<AppState>>, req: Bytes) -> Response<Body> {
    let user_candidate: Result<LoginRequest, _> = serde_json::from_slice(&body);

    if user_candidate.is_err() {
        return respond_with_error(StatusCode::BAD_REQUEST, "something went wrong");
    }

    let user_candidate = user_candidate.unwrap();
    let user = state
        .lock()
        .await
        .db
        .get_user_by_email(&user_candidate.email);
    println!("{:?} -- test", user);
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
