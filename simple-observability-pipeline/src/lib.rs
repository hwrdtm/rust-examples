use std::time::Duration;

use anyhow::Result;
use hyper_util::rt::TokioIo;
use opentelemetry::trace::TraceError;
use opentelemetry::{logs::LogError, metrics::MetricsError};
use opentelemetry_otlp::{ExportConfig, TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

// pub const DEFAULT_SOCK: &str = "/var/run/lit-logging-service-grpc.sock";
pub const DEFAULT_SOCK: &str = "/tmp/proxy-server.sock";

pub async fn init_tonic_exporter_builder() -> Result<TonicExporterBuilder> {
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
    Ok(opentelemetry_otlp::new_exporter()
        .tonic()
        .with_channel(channel)
        .with_export_config(ExportConfig {
            endpoint: "".to_string(),
            protocol: opentelemetry_otlp::Protocol::Grpc,
            timeout: Duration::from_secs(3),
        })) // Then pass it into pipeline builder
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
