use std::time::Duration;

use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::trace::{TraceContextExt, Tracer, TracerProvider};
use opentelemetry::Key;
use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;
use simple_observability_pipeline::{
    create_providers, init_logs, init_metrics, init_tonic_exporter_builder, init_tracer_provider,
};
use tracing::{error, info, instrument, Subscriber};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-client",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::URL_DOMAIN,
            "localhost",
        ),
    ])
});

fn main() -> Result<()> {
    let logging_rt = tokio::runtime::Runtime::new().expect("Failed to create logging runtime");
    let rt_1 = tokio::runtime::Runtime::new().expect("Failed to create runtime 1");
    let rt_2 = tokio::runtime::Runtime::new().expect("Failed to create runtime 2");

    let observability_providers = logging_rt.block_on(async {
        let observability_providers = init_observability().await?;
        Ok::<ObservabilityProviders, anyhow::Error>(observability_providers)
    })?;

    let rt1 = rt_1.spawn(async {
        emit_dummy_events("rt_1".to_string()).await;
        Ok::<(), anyhow::Error>(())
    });

    let rt2 = rt_2.spawn(async {
        emit_dummy_events("rt_2".to_string()).await;
        Ok::<(), anyhow::Error>(())
    });

    // Sleep for 30 seconds
    std::thread::sleep(Duration::from_secs(30));

    observability_providers.shutdown();

    Ok(())
}

async fn emit_dummy_events(thread_name: String) {
    tokio::time::sleep(Duration::from_secs(10)).await;

    let common_scope_attributes = vec![KeyValue::new("scope-key", thread_name.clone())];

    // Send dummy metrics events
    let meter = global::meter_with_version(
        format!("basic-otlp-example-{}", thread_name),
        Some("v1.0"),
        Some("schema_url"),
        Some(common_scope_attributes.clone()),
    );

    let counter = meter
        .u64_counter("test_counter")
        .with_description("a simple counter for demo purposes.")
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

    slow_process().await;

    // Send dummy log event
    info!(name: "my-event", target: "my-target", "[Date: {}] hello from {}. My price is {}", chrono::Utc::now().to_rfc3339(), "apple", 1.99);

    println!("{:?}: Done emitting dummy events", thread_name);

    // sleep for 500 seconds
    tokio::time::sleep(Duration::from_secs(20)).await;
}

#[instrument]
async fn slow_process() {
    tokio::time::sleep(Duration::from_secs(2)).await;
}

async fn init_observability() -> Result<ObservabilityProviders> {
    let (tracing_provider, metrics_provider, subscriber, logger_provider) =
        create_providers(RESOURCE.clone(), false).await?;

    // Set globals
    global::set_tracer_provider(tracing_provider);
    global::set_meter_provider(metrics_provider.clone());
    subscriber.init();

    Ok(ObservabilityProviders::new(
        metrics_provider,
        logger_provider,
    ))
}

#[derive(Default)]
struct ObservabilityProviders {
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<LoggerProvider>,
}

impl ObservabilityProviders {
    fn new(meter_provider: SdkMeterProvider, logger_provider: LoggerProvider) -> Self {
        Self {
            meter_provider: Some(meter_provider),
            logger_provider: Some(logger_provider),
        }
    }

    fn shutdown(self) {
        if let Some(meter_provider) = self.meter_provider {
            if let Err(e) = meter_provider.shutdown() {
                error!("Failed to shutdown metrics provider: {:?}", e);
            }
        }
        if let Some(logger_provider) = self.logger_provider {
            if let Err(e) = logger_provider.shutdown() {
                error!("Failed to shutdown logger provider: {:?}", e);
            }
        }
    }
}
