#![feature(async_closure)]

use std::collections::HashMap;

use log::{debug, info};
use opentelemetry::propagation::Injector;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use retry::call_with_retry;
use tracing::Instrument;
use tracing::info_span;
use tracing::instrument;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

#[tokio::main]
async fn main() -> std::result::Result<(), anyhow::Error> {
    env_logger::init();

    info!("Starting up");

    let subscriber = tracing_subscriber::Registry::default();

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("rust-examples/distributed-tracing-retry".to_string())
        .install_simple()?;
    tracing::subscriber::set_global_default(
        subscriber.with(tracing_opentelemetry::layer().with_tracer(tracer)),
    )
    .expect("setting default subscriber failed");

    let qs = "?hello=world".to_string();


    // TODO: Investigate why the retry doesn't get triggered immediately and the runtime is sleeping for some time.
    {
        call_with_retry(async || make_request(&qs).await)
            .instrument(info_span!("retry_block"))
            .await?;
    }

    {
        call_with_retry(async || make_request(&qs).await)
            .instrument(info_span!("retry_block_v2")) // FIXME: Somehow this doesn't show up.
            .await?;
    }

    Ok(())
}

#[instrument(name = "make_request", skip_all, ret)]
async fn make_request(qs: &String) -> Result<(), anyhow::Error> {
    // Get the OpenTelemetry `Context` via the current `tracing::Span`.
    let cx = tracing::Span::current().context();

    // Initialize the request builder.
    let mut request_builder = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
        .get(format!("http://localhost:8000/hello/world/{}", qs));

    // Initialize the header injector.
    let mut additional_headers = HashMap::new();
    let mut header_injector = HeaderInjector::from(&mut additional_headers);

    // Inject the current context into the request header
    let propagator = TraceContextPropagator::new();
    propagator.inject_context(&cx, &mut header_injector);
    debug!("additional_headers: {:?}", additional_headers);

    // Transfer header information from the carrier into the request builder
    for (key, value) in additional_headers {
        request_builder = request_builder.header(key, value);
    }

    // Send the request
    let res = request_builder.send().await?;

    // Parse the response
    let body = res.text().await?;
    debug!("body: {}", body);

    Ok(())
}

struct HeaderInjector<'a> {
    header_map: &'a mut HashMap<String, String>,
}
impl<'a> From<&'a mut HashMap<String, String>> for HeaderInjector<'a> {
    fn from(header_map: &'a mut HashMap<String, String>) -> Self {
        HeaderInjector { header_map }
    }
}

impl<'a> Injector for HeaderInjector<'a> {
    fn set(&mut self, key: &str, value: String) {
        println!("set {} {}", key, value);
        self.header_map.insert(key.to_string(), value);
    }
}
