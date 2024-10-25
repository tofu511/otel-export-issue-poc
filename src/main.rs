use opentelemetry::global::shutdown_tracer_provider;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Tracer;
use opentelemetry_sdk::Resource;
use tonic::metadata::MetadataMap;
use tracing::instrument;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Registry;

fn init_subscriber() {
    let otel = tracing_opentelemetry::layer().with_tracer(init_tracer());

    Registry::default()
        .with(tracing_subscriber::fmt::layer().with_span_events(FmtSpan::NEW | FmtSpan::EXIT))
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
                .with_metadata(map)
                .with_endpoint("https://api.honeycomb.io/api/traces")
                .with_timeout(std::time::Duration::from_secs(5)),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap()
        .tracer("otel-export-issue-poc")
}

#[instrument]
fn foo() {
    tracing::info!("inside foo");
    bar();
}

#[instrument]
fn bar() {
    tracing::info!("inside bar");
}

#[tokio::main]
async fn main() {
    init_subscriber();
    foo();

    shutdown_tracer_provider();
}
