use std::{net::TcpListener, sync::Arc, time::Duration};

use crate::{
    domain::StorageDetails,
    routes::{health_check, upload},
};
use axum::{
    body::BoxBody,
    response::Response,
    routing::{get, post},
    Extension, Router,
};
use hyper::{Body, Request};
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;
use uuid::Uuid;

pub async fn app(listener: TcpListener, storage_details: StorageDetails) {
    let router = Router::new()
        .route("/health_check", get(health_check))
        .route("/upload", post(upload))
        .layer(Extension(Arc::new(storage_details)));

    let router = add_tracing_middleware(router);

    axum::Server::from_tcp(listener)
        .unwrap()
        .serve(router.into_make_service())
        .await
        .unwrap();
}

fn add_tracing_middleware(router: Router) -> Router {
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|_request: &Request<Body>| {
            let request_id = Uuid::new_v4().to_string();
            tracing::info_span!("http-request", %request_id)
        })
        .on_request(|request: &Request<Body>, _span: &Span| {
            tracing::info!("request: {} {}", request.method(), request.uri().path())
        })
        .on_response(
            |response: &Response<BoxBody>, latency: Duration, _span: &Span| {
                tracing::info!("response: {} {:?}", response.status(), latency)
            },
        )
        .on_failure(
            |error: ServerErrorsFailureClass, _latency: Duration, _span: &Span| {
                tracing::error!("error: {}", error)
            },
        );
    router.layer(tracing_layer)
}
