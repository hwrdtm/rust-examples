use std::time::Duration;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use simple_observability_pipeline::DEFAULT_SOCK;
use tokio::net::UnixStream;
use tonic::transport::{Endpoint, Uri};
use tower::service_fn;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tonic will ignore this uri because uds do not use it
    // if the connector does use the uri it will be provided
    // as the request to the `MakeConnection`.
    let channel = Endpoint::try_from("http://127.0.0.1:4371")?
        .connect_with_connector(service_fn(|_: Uri| {
            // Connect to a Uds socket
            UnixStream::connect(DEFAULT_SOCK)
        }))
        .await?;

    // First, create a OTLP exporter builder. Configure it as you need.
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_channel(channel)
        .with_export_config(ExportConfig {
            endpoint: "".to_string(),
            protocol: opentelemetry_otlp::Protocol::Grpc,
            timeout: Duration::from_secs(3),
        }); // Then pass it into pipeline builder
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(otlp_exporter)
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()?;

    global::set_meter_provider(meter_provider.clone());

    // get a meter from a provider
    let meter = global::meter("my_service");

    // create an instrument
    let counter = meter.u64_counter("my_counter").init();

    // record a measurement
    counter.add(1, &[KeyValue::new("http.client_ip", "83.164.160.102")]);

    if let Err(e) = meter_provider.force_flush() {
        eprintln!("Failed to flush metrics: {}", e);
    }

    println!("Done!");

    Ok(())
}
