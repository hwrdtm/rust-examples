use anyhow::Result;
use once_cell::sync::Lazy;
use simple_observability_pipeline::{
    create_providers,
    opentelemetry::{global, KeyValue},
    opentelemetry_sdk::{
        logs::LoggerProvider, metrics::SdkMeterProvider, propagation::TraceContextPropagator,
        Resource,
    },
};
use tracing::{error, instrument, Instrument};
use tracing_channels::{new_bounded_channel, new_unbounded_channel, TracedReceiver, TracedSender};
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

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
    let observability_rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    let observability_providers = observability_rt.block_on(async {
        let observability_providers = init_observability().await?;
        Ok::<ObservabilityProviders, anyhow::Error>(observability_providers)
    })?;

    let main_rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");
    main_rt.block_on(async {
        oneshot_channel_example().await;

        forever_loop_example().await;

        // Sleep for 10 seconds to export traces
        tokio::time::sleep(std::time::Duration::from_secs(100)).await;
    });

    println!("shutting down observability providers");
    observability_providers.shutdown();

    Ok(())
}

/// Simulate producer / consumer workers in separate tasks.
///
/// We intentionally do NOT instrument this function to have each send be the root of each trace.
async fn forever_loop_example() {
    // Create new flume unbounded channel.
    let (tx, rx) = new_unbounded_channel::<bool>();

    // Spawn a new task that will send a value to the channel every 100ms.
    tokio::spawn(async move { send_to_channel_continuously(tx).await });

    // Spawn a new task that will receive a value from the channel.
    tokio::spawn(async move { receive_from_channel_continuously(rx).await });
}

async fn send_to_channel_continuously(tx: TracedSender<bool>) {
    loop {
        tx.send_async(true).await.expect("Failed to send value");
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
}

async fn receive_from_channel_continuously(rx: TracedReceiver<bool>) {
    while let Ok((_d, span)) = rx.recv_async().await {
        async {
            // Do work
            tracing::info!("received value");

            doing_some_work().await;
        }
        .instrument(span)
        .await;
    }
}

/// One shot channel example.
#[instrument(level = "info")]
async fn oneshot_channel_example() {
    tracing::info!("outer trace");

    // Create new flume bounded channel.
    let (tx, rx) = new_bounded_channel::<bool>(0);

    // Spawn a new task that will send a value to the channel.
    let send_task = tracing::span!(tracing::Level::INFO, "send_task");
    tokio::spawn(
        async move {
            tx.send_async(true).await.expect("Failed to send value");
        }
        .instrument(send_task),
    );

    // Spawn a new task that will receive a value from the channel.
    tokio::spawn(async move {
        let (_d, span) = rx.recv_async().await.expect("Failed to receive value");

        async {
            // Do work
            tracing::info!("received value");

            doing_some_work().await;
        }
        .instrument(span)
    });

    inner_trace();

    // Sleep for 1s
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
}

#[instrument]
async fn doing_some_work() {
    tracing::info!("doing some work");

    // Sleep for 1.3333s
    tokio::time::sleep(std::time::Duration::from_secs_f32(1.3333)).await;
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
