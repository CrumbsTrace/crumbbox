use crumbbox::{
    configuration::Settings,
    domain::StorageDetails,
    startup::app,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use std::net::{SocketAddr, TcpListener};

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter = String::from("info");

    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(default_filter, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(default_filter, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: SocketAddr,
    pub storage_path: String,
}

impl TestApp {
    pub fn addr(&self) -> String {
        format!("http://{}", self.address)
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = {
        let mut config = Settings::get_configuration().expect("Failed to get configuration");
        config.application.port = 0;
        config.application.storage_path = ".crumbbox/test/".to_string();
        config
    };

    let listener = TcpListener::bind(
        format!("127.0.0.1:{}", config.application.port)
            .parse::<SocketAddr>()
            .unwrap(),
    )
    .unwrap();
    let address = listener.local_addr().unwrap();
    let storage_details = StorageDetails {
        path: config.application.storage_path.clone(),
    };

    let _ = tokio::spawn(app(listener, storage_details));

    TestApp {
        address,
        storage_path: config.application.storage_path,
    }
}
