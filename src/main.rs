use crumbbox::{
    configuration::Settings,
    startup::app,
    telemetry::{get_subscriber, init_subscriber},
};
use std::net::{SocketAddr, TcpListener};

#[tokio::main]
async fn main() {
    let config = Settings::get_configuration().expect("Failed to load configuration");

    let subscriber = get_subscriber("info".to_string(), std::io::stdout);
    init_subscriber(subscriber);

    let address = format!("{}:{}", config.application.host, config.application.port)
        .parse::<SocketAddr>()
        .expect("Failed to parse address");

    let listener = TcpListener::bind(address).unwrap();

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    app(listener).await;
}
