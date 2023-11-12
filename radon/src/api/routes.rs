#[derive(Deserialize)]
pub struct RegisterRequest {
    username: String,
}

async fn register(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RegisterRequest>,
) -> Result<Response<Body>, Infallible> {
    let mut user_set = state.user_set.lock().unwrap();

    if user_set.contains(&payload.username) {
        let response = Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("Username already taken".into())
            .unwrap();
        return Ok(response);
    } else {
        user_set.insert(payload.username);
        let response = Response::builder()
            .status(StatusCode::CREATED)
            .body("User registered".into())
            .unwrap();
        return Ok(response);
    }
}
