use std::time::Duration;

use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // First, create a OTLP exporter builder. Configure it as you need.
    let otlp_exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .with_export_config(ExportConfig {
            endpoint: "http://127.0.0.1:4317".to_string(),
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
