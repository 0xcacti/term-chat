pub mod error;
pub mod users;

use axum::{
    http::{header::CONTENT_TYPE, HeaderName, Method, Response, StatusCode},
    Extension, Json, Router, Server,
};
use sqlx::PgPool;
use tower_http::cors::{Any, CorsLayer};

use crate::config::ServerConfig;

#[derive(Debug, Clone)]
pub struct AppState {
    pub config: ServerConfig,
    pub db: PgPool,
}

pub async fn run(state: AppState) {
    let app = routes(&state);
    let addr = format!("127.0.0.1:{}", state.config.port).parse().unwrap();
    println!("Listening on {}", addr);
    let server = Server::bind(&addr).serve(app.into_make_service());
    server.await.unwrap();
}

pub fn routes(state: &AppState) -> Router {
    let cors = get_cors();

    Router::new()
        .merge(users::router())
        .layer(Extension(state))
        .layer(cors)
}

pub fn get_cors() -> CorsLayer {
    let cors = CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_origin(Any)
        .allow_headers([
            CONTENT_TYPE,
            HeaderName::from_static("x-requested-with"),
            HeaderName::from_static("cache-control"),
            HeaderName::from_static("authorization"),
        ]);
    cors
}
