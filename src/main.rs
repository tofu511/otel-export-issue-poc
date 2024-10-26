use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{BatchConfig, BatchConfigBuilder, Tracer};
use opentelemetry_sdk::Resource;
use std::time::Duration;
use tonic::metadata::MetadataMap;
use tracing::instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

fn init_subscriber() {
    let otel = tracing_opentelemetry::layer().with_tracer(init_tracer());

    Registry::default()
        // .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::EXIT))
        .with(tracing_subscriber::filter::LevelFilter::DEBUG)
        .with(otel)
        .init();
}

fn init_tracer() -> Tracer {
    let honeycomb_key =
        std::env::var("HONEYCOMB_API_KEY").expect("`HONEYCOMB_API_KEY` must be set");
    let mut map = MetadataMap::with_capacity(1);
    map.insert("x-honeycomb-team", honeycomb_key.try_into().unwrap());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(opentelemetry_sdk::trace::Config::default().with_resource(
            Resource::new(vec![KeyValue::new("service.name", "otel-export-issue-poc")]),
        ))
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_tls_config(tonic::transport::ClientTlsConfig::new().with_native_roots())
                .with_metadata(map)
                .with_endpoint("https://api.honeycomb.io/api/traces")
                .with_timeout(std::time::Duration::from_secs(5)),
        )
        .with_batch_config(
            BatchConfigBuilder::default()
                .with_scheduled_delay(Duration::from_millis(100))
                .build(),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
        .tracer("otel-export-issue-poc")
}

#[instrument]
async fn foo() {
    tracing::info!("inside foo");
    bar().await;
}

#[instrument]
async fn bar() {
    tracing::info!("inside bar");
}

#[tokio::main]
async fn main() {
    init_subscriber();
    foo().await;

    tokio::time::sleep(Duration::from_secs(2)).await;
    shutdown_tracer_provider();
}
