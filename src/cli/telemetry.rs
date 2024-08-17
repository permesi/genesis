use anyhow::Result;
use opentelemetry::{global, trace::TracerProvider, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{runtime::Tokio, trace::Config, Resource};
use std::time::Duration;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

/// Start the telemetry layer
/// # Errors
/// Will return an error if the vault role-id or vault url is not provided
pub fn init(verbosity_level: tracing::Level) -> Result<()> {
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_timeout(Duration::from_secs(3));

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(otlp_exporter)
        .with_trace_config(Config::default().with_resource(Resource::new(vec![
            KeyValue::new("service.name", env!("CARGO_PKG_NAME")),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])))
        .install_batch(Tokio)?;

    let tracer = provider
        .tracer_builder(env!("CARGO_PKG_NAME"))
        .with_version(env!("CARGO_PKG_VERSION"))
        .build();

    global::set_tracer_provider(provider);

    let otel_layer = OpenTelemetryLayer::new(tracer);

    let fmt_layer = fmt::layer()
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_target(false)
        .pretty();

    // RUST_LOG=
    let env_filter = EnvFilter::builder()
        .with_default_directive(verbosity_level.into())
        .from_env_lossy();

    let subscriber = Registry::default()
        .with(fmt_layer)
        .with(otel_layer)
        .with(env_filter);

    Ok(tracing::subscriber::set_global_default(subscriber)?)
}
