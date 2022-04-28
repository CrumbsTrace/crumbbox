use crumbbox::startup::app;
use std::net::{SocketAddr, TcpListener};

pub struct TestApp {
    pub address: SocketAddr,
}

impl TestApp {
    pub fn addr(&self) -> String {
        format!("http://{}", self.address)
    }
}

pub async fn spawn_app() -> TestApp {
    let listener = TcpListener::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap()).unwrap();
    let address = listener.local_addr().unwrap();

    let _ = tokio::spawn(app(listener));

    TestApp { address }
}
