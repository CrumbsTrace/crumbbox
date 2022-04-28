use crumbbox::startup::app;
use std::net::{SocketAddr, TcpListener};
use tracing_subscriber::{prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listener = TcpListener::bind("127.0.0.1:3000".parse::<SocketAddr>().unwrap()).unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());

    app(listener).await;
}
