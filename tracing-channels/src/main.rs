use std::collections::HashMap;

use anyhow::Result;
use once_cell::sync::Lazy;
use simple_observability_pipeline::{
    create_providers,
    opentelemetry::{
        global,
        propagation::{Extractor, Injector},
        KeyValue,
    },
    opentelemetry_sdk::{
        logs::LoggerProvider, metrics::SdkMeterProvider, propagation::TraceContextPropagator,
        trace::Span, Resource,
    },
};
use tracing::{error, instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::util::SubscriberInitExt;

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

pub struct ChannelData<T> {
    metadata: HashMap<String, String>,
    data: T,
}

pub struct ChannelMetadata<'a>(&'a mut HashMap<String, String>);

impl<'a> Injector for ChannelMetadata<'a> {
    fn set(&mut self, key: &str, value: String) {
        self.0.insert(key.to_lowercase(), value);
    }
}

impl<'a> Extractor for ChannelMetadata<'a> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(|k| k.as_str()).collect()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let observability_rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let observability_providers = observability_rt.block_on(async {
        let observability_providers = init_observability().await?;
        Ok::<ObservabilityProviders, anyhow::Error>(observability_providers)
    })?;

    let main_rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    main_rt.block_on(async {
        outer_trace().await;

        // Sleep for 10 seconds to export traces
        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    });

    println!("shutting down observability providers");
    observability_providers.shutdown();

    Ok(())
}

#[instrument(level = "info")]
async fn outer_trace() {
    tracing::info!("outer trace");

    // Create new flume bounded channel.
    let (tx, rx) = flume::bounded::<ChannelData<bool>>(10);

    // Spawn a new task that will send a value to the channel.
    let task_span = tracing::span!(tracing::Level::INFO, "task");
    tokio::spawn(async move {
        let _guard = task_span.enter();

        // Inject tracing context into metadata.
        let mut metadata = HashMap::new();
        let cx = tracing::Span::current().context();
        global::get_text_map_propagator(|propagator| {
            propagator.inject_context(&cx, &mut ChannelMetadata(&mut metadata))
        });

        tx.send_async(ChannelData {
            metadata,
            data: true,
        })
        .await
        .unwrap();
    });

    // Spawn a new task that will receive a value from the channel.
    tokio::spawn(async move {
        let mut value = rx.recv_async().await.expect("Failed to receive value");

        // Extract the propagated tracing context from the incoming request headers.
        let parent_cx = global::get_text_map_propagator(|propagator| {
            propagator.extract(&ChannelMetadata(&mut value.metadata))
        });

        // Initialize a new span with the extracted tracing context as the parent.
        let info_span = tracing::info_span!("doing_some_work",);
        info_span.set_parent(parent_cx);

        // Do work
        tracing::info!("received value");

        // Sleep for 1.3333s
        tokio::time::sleep(std::time::Duration::from_secs_f32(1.3333)).await;
    });

    inner_trace();

    // Sleep for 1s
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[instrument]
fn inner_trace() {
    tracing::info!("inner trace");
}

async fn init_observability() -> Result<ObservabilityProviders> {
    let (tracing_provider, metrics_provider, subscriber, logger_provider) =
        create_providers(RESOURCE.clone(), false, false).await?;

    // Set globals
    global::set_text_map_propagator(TraceContextPropagator::new());
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
