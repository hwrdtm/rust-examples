// RUST_LOG=debug cargo run

#[macro_use]
extern crate rocket;

use std::collections::HashMap;

use anyhow::anyhow;
use log::{debug, info};
use opentelemetry::{propagation::Extractor, sdk::propagation::TraceContextPropagator};
use rocket::serde::json::json;
use rocket::{http::Status, request::Outcome, response::status, routes, serde::json::Value};
use tracing::{info_span, instrument, Instrument};
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::prelude::*;

use opentelemetry::propagation::TextMapPropagator;

// Global mutable, thread-safe counter.
use std::sync::atomic::{AtomicUsize, Ordering};

static TIMEOUTSUCCESS: AtomicUsize = AtomicUsize::new(0);
static TIMEOUTERROR: AtomicUsize = AtomicUsize::new(0);

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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

#[get("/world?<mode>")]
async fn world(tracing_span: TracingSpan, mode: &str) -> status::Custom<Value> {
    if mode == "timeoutsuccess" {
        let _ = timeoutsuccess()
            .instrument(tracing_span.span().to_owned())
            .await;
    } else if mode == "timeouterror" {
        if let Err(e) = timeouterror()
            .instrument(tracing_span.span().to_owned())
            .await
        {
            error!("timeouterror: {:?}", e);
            return status::Custom(
                Status::InternalServerError,
                json!({
                    "error": e.to_string(),
                }),
            );
        }
    } else if mode == "error" {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return status::Custom(
            Status::InternalServerError,
            json!({
                "error": "customerror",
            }),
        );
    } else if mode == "success" {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    status::Custom(
        Status::Ok,
        json!({
            "message": "Life is good.",
        }),
    )
}

#[instrument(name = "timeoutsuccess", skip_all, ret)]
async fn timeoutsuccess() -> Result<(), anyhow::Error> {
    info!("[timeoutsuccess]");

    let counter = TIMEOUTSUCCESS.fetch_add(1, Ordering::SeqCst);

    if counter < 1 {
        debug!("Sleeping for 10s");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    } else {
        debug!("Sleeping for 1s");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    Ok(())
}

#[instrument(name = "timeouterror", skip_all, ret)]
async fn timeouterror() -> Result<(), anyhow::Error> {
    info!("[timeouterror]");

    let counter = TIMEOUTERROR.fetch_add(1, Ordering::SeqCst);

    if counter < 1 {
        debug!("Sleeping for 10s");
        tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    } else {
        debug!("Sleeping for 1s");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        return Err(anyhow!("timeouterror"));
    }

    Ok(())
}
