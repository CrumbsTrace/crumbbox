use tracing::{subscriber::set_global_default, Subscriber};
use tracing_log::LogTracer;
use tracing_subscriber::{
    fmt::{format::FmtSpan, MakeWriter},
    layer::SubscriberExt,
    EnvFilter, Registry,
};

pub fn get_subscriber<Sink>(default_env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = std::env::var("RUST_LOG").unwrap_or(default_env_filter);
    Registry::default().with(EnvFilter::new(env_filter)).with(
        tracing_subscriber::fmt::layer()
            .with_writer(sink)
            .with_span_events(FmtSpan::CLOSE),
    )
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to initialize logger");
    set_global_default(subscriber).expect("Failed to set global subscriber");
}
