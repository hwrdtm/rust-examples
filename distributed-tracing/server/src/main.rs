// RUST_LOG=debug cargo run

#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use log::{debug, info};
use opentelemetry::{propagation::Extractor, sdk::propagation::TraceContextPropagator};
use rocket::{request::Outcome, routes};
use tracing::{info_span, instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

use opentelemetry::propagation::TextMapPropagator;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    info!("Starting up");

    let subscriber = tracing_subscriber::Registry::default();

    let tracer = opentelemetry_jaeger::new_agent_pipeline()
        .with_service_name("rust-examples/distributed-tracing".to_string())
        .install_simple()?;
    tracing::subscriber::set_global_default(
        subscriber.with(tracing_opentelemetry::layer().with_tracer(tracer)),
    )
    .expect("setting default subscriber failed");

    let _rocket = rocket::build()
        .mount("/hello", routes![world])
        .launch()
        .await?;

    Ok(())
}

#[derive(Debug)]
pub struct TracingSpan {
    span: tracing::Span,
}

impl TracingSpan {
    pub fn span(&self) -> &tracing::Span {
        &self.span
    }

    pub fn new(span: tracing::Span) -> Self {
        Self { span }
    }
}

struct HashMapExtractor<'a> {
    headers: &'a HashMap<String, String>,
}

impl<'a> From<&'a HashMap<String, String>> for HashMapExtractor<'a> {
    fn from(headers: &'a HashMap<String, String>) -> Self {
        HashMapExtractor { headers }
    }
}

impl<'a> Extractor for HashMapExtractor<'a> {
    fn get(&self, key: &str) -> Option<&'a str> {
        self.headers.get(key).map(|v| v.as_str())
    }

    fn keys(&self) -> Vec<&str> {
        self.headers.keys().map(|v| v.as_str()).collect()
    }
}

#[rocket::async_trait]
impl<'r> rocket::request::FromRequest<'r> for TracingSpan {
    type Error = ();

    async fn from_request(
        req: &'r rocket::Request<'_>,
    ) -> rocket::request::Outcome<Self, Self::Error> {
        debug!("req headers: {:?}", req.headers());

        // Extract the propagated context
        let propagator = TraceContextPropagator::new();
        // Initialize some container to hold the header information.
        let mut carrier = HashMap::new();
        // Transfer header information from request to carrier.
        for header in req.headers().iter() {
            carrier.insert(header.name().to_string(), header.value().to_string());
        }
        // Extract the context from the carrier
        let context = propagator.extract(&HashMapExtractor::from(&carrier));

        // Initialize a new span with the propagated context as the parent
        let req_method = req.method();
        let req_path = req.uri().path();
        let new_span = info_span!(
            "handle_request",
            method = req_method.as_str(),
            path = req_path.to_string(),
        );
        new_span.set_parent(context);

        Outcome::Success(TracingSpan { span: new_span })
    }
}

#[get("/world")]
async fn world(tracing_span: TracingSpan) -> &'static str {
    // Call something
    let _ = something().instrument(tracing_span.span().to_owned()).await;
    "Hello, world!"
}

#[instrument(name = "something", fields(thing=2) skip_all, ret)]
async fn something() -> Result<(), anyhow::Error> {
    info!("[something]");

    // Sleep for 1s before calling another function.
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    // Call something_inner
    something_inner().await?;

    Ok(())
}

#[instrument(name = "something_inner", skip_all, ret)]
async fn something_inner() -> Result<(), anyhow::Error> {
    info!("[something_inner]");

    // Sleep for 2s
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    Ok(())
}
