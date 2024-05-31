use std::time::Duration;

use once_cell::sync::Lazy;
use opentelemetry::metrics::Unit;
use opentelemetry::trace::{TraceContextExt, TraceError, Tracer, TracerProvider};
use opentelemetry::Key;
use opentelemetry::{global, logs::LogError, metrics::MetricsError, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExportConfig, TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use simple_observability_pipeline::DEFAULT_SOCK;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;
use tracing::info;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![KeyValue::new(
        opentelemetry_semantic_conventions::resource::SERVICE_NAME,
        "basic-otlp-example",
    )])
});

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the tracing pipeline
    let tracing_provider = init_tracer_provider(init_metrics_exporter_builder().await?)?
        .provider()
        .expect("Tracer provider not set.");
    global::set_tracer_provider(tracing_provider.clone());

    // Initialize the metrics pipeline
    let meter_provider = init_metrics(init_metrics_exporter_builder().await?)?;
    global::set_meter_provider(meter_provider.clone());

    // Initialize the logs pipeline
    let logger_provider = init_logs(init_metrics_exporter_builder().await?)?;

    // Create a new OpenTelemetryTracingBridge using the above LoggerProvider.
    let layer = OpenTelemetryTracingBridge::new(&logger_provider);

    // Add a tracing filter to filter events from crates used by opentelemetry-otlp.
    // The filter levels are set as follows:
    // - Allow `info` level and above by default.
    // - Restrict `hyper`, `tonic`, and `reqwest` to `error` level logs only.
    // This ensures events generated from these crates within the OTLP Exporter are not looped back,
    // thus preventing infinite event generation.
    // Note: This will also drop events from these crates used outside the OTLP Exporter.
    // For more details, see: https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let filter = EnvFilter::new("info")
        .add_directive("hyper=error".parse().unwrap())
        .add_directive("tonic=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    tracing_subscriber::registry()
        .with(filter)
        .with(layer)
        .init();

    // Send dummy metrics events
    let common_scope_attributes = vec![KeyValue::new("scope-key", "scope-value")];
    let meter = global::meter_with_version(
        "basic",
        Some("v1.0"),
        Some("schema_url"),
        Some(common_scope_attributes.clone()),
    );

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
        .with_unit(Unit::new("my_unit"))
        .init();
    for _ in 0..10 {
        counter.add(1, &[KeyValue::new("http.client_ip", "83.164.160.102")]);
    }

    // Send dummy trace events
    let tracer = global::tracer_provider()
        .tracer_builder("basic")
        .with_attributes(common_scope_attributes.clone())
        .build();

    tracer.in_span("Main operation", |cx| {
        let span = cx.span();
        span.add_event(
            "Nice operation!".to_string(),
            vec![Key::new("bogons").i64(100)],
        );
        span.set_attribute(KeyValue::new("another.key", "yes"));

        info!(name: "my-event-inside-span", target: "my-target", "hello from {}. My price is {}. I am also inside a Span!", "banana", 2.99);

        tracer.in_span("Sub operation...", |cx| {
            let span = cx.span();
            span.set_attribute(KeyValue::new("another.key", "yes"));
            span.add_event("Sub span event", vec![]);
        });
    });

    // Send dummy log event
    info!(name: "my-event", target: "my-target", "hello from {}. My price is {}", "apple", 1.99);

    // Shutdown the pipelines, which will also cause the exporters to flush any remaining data.
    global::shutdown_tracer_provider();
    meter_provider.shutdown()?;
    logger_provider.shutdown()?;

    println!("Done!");

    Ok(())
}

async fn init_metrics_exporter_builder() -> Result<TonicExporterBuilder, Box<dyn std::error::Error>>
{
    // Tonic will ignore this uri because uds do not use it
    // if the connector does use the uri it will be provided
    // as the request to the `MakeConnection`.
    let channel = Endpoint::try_from("http://127.0.0.1:4371")
        .map_err(|e| MetricsError::Other(e.to_string()))?
        .connect_with_connector(service_fn(|_: Uri| {
            // Connect to a Uds socket
            UnixStream::connect(DEFAULT_SOCK)
        }))
        .await
        .map_err(|e| MetricsError::Other(e.to_string()))?;

    // First, create a OTLP exporter builder. Configure it as you need.
    Ok(opentelemetry_otlp::new_exporter()
        .tonic()
        .with_channel(channel)
        .with_export_config(ExportConfig {
            endpoint: "".to_string(),
            protocol: opentelemetry_otlp::Protocol::Grpc,
            timeout: Duration::from_secs(3),
        })) // Then pass it into pipeline builder
}

fn init_tracer_provider(
    metrics_exporter_builder: TonicExporterBuilder,
) -> Result<sdktrace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(metrics_exporter_builder)
        .with_trace_config(sdktrace::config().with_resource(RESOURCE.clone()))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

fn init_metrics(
    metrics_exporter_builder: TonicExporterBuilder,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(metrics_exporter_builder)
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .with_resource(RESOURCE.clone())
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()
}

fn init_logs(
    metrics_exporter_builder: TonicExporterBuilder,
) -> Result<opentelemetry_sdk::logs::LoggerProvider, LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(metrics_exporter_builder)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}
