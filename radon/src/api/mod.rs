pub mod error;
pub mod users;

use axum::{
    body::Body,
    extract::State,
    http::{header::CONTENT_TYPE, HeaderName, Method, Response, StatusCode},
    response::Html,
    routing::{get, post},
    Extension, Json, Router, Server,
};
use sqlx::PgPool;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

use crate::config::ServerConfig;

pub async fn run(db: PgPool) {
    let app = routes(db);
    let addr = "127.0.0.1:8081".parse().unwrap();
    println!("Listening on {}", addr);
    let server = Server::bind(&addr).serve(app.into_make_service());
    server.await.unwrap();
}

pub fn routes(db: PgPool) -> Router {
    let cors = get_cors();

    Router::new()
        .merge(user::router())
        .layer(Extension(db))
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
}
