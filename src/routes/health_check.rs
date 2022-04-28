use axum::{http::StatusCode, response::IntoResponse};

#[tracing::instrument(name = "Check the health of the server")]
pub async fn health_check() -> impl IntoResponse {
    tracing::info!("A health check was requested");
    StatusCode::OK
}
