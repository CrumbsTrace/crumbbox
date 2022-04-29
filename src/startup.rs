use std::net::TcpListener;

use crate::routes::{health_check, upload};
use axum::{
    routing::{get, post},
    Extension, Router,
};

pub async fn app(listener: TcpListener, storage_path: String) {
    let router = Router::new()
        .route("/health_check", get(health_check))
        .route("/upload", post(upload))
        .layer(Extension(storage_path));

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(router.into_make_service())
        .await
        .unwrap();
}
