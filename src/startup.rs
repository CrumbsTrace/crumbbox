use std::net::TcpListener;

use crate::routes::health_check;
use axum::{routing::get, Router};

pub async fn app(listener: TcpListener) {
    let router = Router::new().route("/health_check", get(health_check));

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(router.into_make_service())
        .await
        .unwrap();
}
