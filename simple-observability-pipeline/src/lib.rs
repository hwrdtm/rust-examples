use std::time::Duration;

use anyhow::Result;
use hyper_util::rt::TokioIo;
use opentelemetry::trace::{TraceError, TracerProvider};
use opentelemetry::{logs::LogError, metrics::MetricsError};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{ExportConfig, TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tokio::net::UnixStream;
use tonic::metadata::MetadataValue;
use tonic::transport::{Endpoint, Uri};
use tonic::{Request, Status};
use tower::service_fn;
use tracing::Subscriber;
use tracing_opentelemetry::{layer, MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

// Re-exports
pub use opentelemetry;
pub use opentelemetry_sdk;

pub const DEFAULT_SOCK: &str = "/tmp/proxy-server.sock";

/// If `exporting_from_logging_service` is true, the exporter will be configured to intercept
/// requests to add additional headers (extensions) to the request.
pub async fn init_tonic_exporter_builder(
    use_proxy: bool,
    exporting_from_logging_service: bool,
) -> Result<TonicExporterBuilder> {
    let mut exporter = if use_proxy {
        // Tonic will ignore this uri because uds do not use it
        // if the connector does use the uri it will be provided
        // as the request to the `MakeConnection`.
        let service_fn = service_fn(|_: Uri| async {
            // Try connecting up to 5 times
            for _ in 0..5 {
                match UnixStream::connect(DEFAULT_SOCK).await {
                    Ok(stream) => return Ok(TokioIo::new(stream)),
                    Err(e) => {
                        println!("Failed to connect to UDS socket: {}", e);

                        // Wait for 1 seconds before retrying.
                        tokio::time::sleep(Duration::from_secs(1)).await;
                    }
                }
            }

            Err(MetricsError::Other(
                "Failed to connect to UDS socket".to_string(),
            ))
        });
        
        let channel = Endpoint::try_from("http://127.0.0.1:4371")
            .map_err(|e| MetricsError::Other(e.to_string()))?
            .connect_with_connector(service_fn)
            .await
            .map_err(|e| MetricsError::Other(e.to_string()))?;

        // First, create a OTLP exporter builder. Configure it as you need.
        opentelemetry_otlp::new_exporter()
            .tonic()
            .with_channel(channel)
            .with_export_config(ExportConfig {
                endpoint: "".to_string(),
                protocol: opentelemetry_otlp::Protocol::Grpc,
                timeout: Duration::from_secs(3),
            })
    } else {
        opentelemetry_otlp::new_exporter().tonic()
    };

    if exporting_from_logging_service {
        exporter = exporter.with_interceptor(intercept);
    }

    Ok(exporter)
}

// An interceptor function.
fn intercept(mut req: Request<()>) -> Result<Request<()>, Status> {
    req.metadata_mut()
        .insert("x-origin", MetadataValue::from_static("proxy-server"));
    Ok(req)
}

pub fn init_tracer_provider(
    tonic_exporter_builder: TonicExporterBuilder,
    resource: Resource,
) -> Result<sdktrace::TracerProvider, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(tonic_exporter_builder)
        .with_trace_config(sdktrace::config().with_resource(resource))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

pub fn init_metrics(
    tonic_exporter_builder: TonicExporterBuilder,
    resource: Resource,
) -> Result<opentelemetry_sdk::metrics::SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(tonic_exporter_builder)
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .with_resource(resource)
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()
}

pub fn init_logs(
    tonic_exporter_builder: TonicExporterBuilder,
    resource: Resource,
) -> Result<opentelemetry_sdk::logs::LoggerProvider, LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_resource(resource)
        .with_exporter(tonic_exporter_builder)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}

pub async fn create_providers(
    resource: Resource,
    use_proxy: bool,
    exporting_from_logging_service: bool,
) -> Result<(
    opentelemetry_sdk::trace::TracerProvider,
    SdkMeterProvider,
    impl Subscriber,
    LoggerProvider,
)> {
    // Initialize the tracing pipeline
    let tracing_provider = init_tracer_provider(
        init_tonic_exporter_builder(use_proxy, exporting_from_logging_service).await?,
        resource.clone(),
    )?;
    let tracer = tracing_provider.tracer("basic-tracer");

    // Initialize the metrics pipeline
    let meter_provider = init_metrics(
        init_tonic_exporter_builder(use_proxy, exporting_from_logging_service).await?,
        resource.clone(),
    )?;

    // Initialize the logs pipeline
    let logger_provider = init_logs(
        init_tonic_exporter_builder(use_proxy, exporting_from_logging_service).await?,
        resource.clone(),
    )?;

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
        .add_directive("tower=error".parse().unwrap())
        .add_directive("h2=error".parse().unwrap())
        .add_directive("reqwest=error".parse().unwrap());

    let sub = tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .with(layer)
        .with(MetricsLayer::new(meter_provider.clone()))
        .with(OpenTelemetryLayer::new(tracer));

    Ok((tracing_provider, meter_provider, sub, logger_provider))
}
