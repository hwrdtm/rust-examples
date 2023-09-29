#![feature(async_closure)]

use core::fmt;
use std::collections::HashMap;

use log::error;
use log::{debug, info};
use opentelemetry::propagation::Injector;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use retry::call_with_retry_condition;
use tracing::info_span;
use tracing::instrument;
use tracing::Instrument;
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

    let first_qs = "?mode=timeoutsuccess".to_string();
    let second_qs = "?mode=timeouterror".to_string();
    let third_qs = "?mode=error".to_string();
    let fourth_qs = "?mode=success".to_string();

    // TODO: Investigate why the retry doesn't get triggered immediately and the runtime is sleeping for some time.
    // Can consider using https://github.com/tokio-rs/console to help profile.
    {
        info!("\n\nFirst case");
        let res = call_with_retry_condition(
            async || make_request(&first_qs).await,
            |e: &AppError| {
                if e.is_timeout() {
                    debug!("Request timed out. Retrying...");
                    true
                } else {
                    debug!("Request failed.");
                    false
                }
            },
        )
        .instrument(info_span!("retry_block_first"))
        .await;
        assert!(res.is_ok());
        let res = res.unwrap();
        info!("res: {:?}", res);
    }

    {
        info!("\n\nSecond case");
        let res = call_with_retry_condition(
            async || make_request(&second_qs).await,
            |e: &AppError| {
                if e.is_timeout() {
                    debug!("Request timed out. Retrying...");
                    true
                } else {
                    debug!("Request failed.");
                    false
                }
            },
        )
        .instrument(info_span!("retry_block_second"))
        .await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        error!("err: {:?}", err);
    }

    {
        info!("\n\nThird case");
        let res = call_with_retry_condition(
            async || make_request(&third_qs).await,
            |e: &AppError| {
                if e.is_timeout() {
                    debug!("Request timed out. Retrying...");
                    true
                } else {
                    debug!("Request failed.");
                    false
                }
            },
        )
        .instrument(info_span!("retry_block_third"))
        .await;
        assert!(res.is_err());
        let err = res.unwrap_err();
        error!("err: {:?}", err);
    }

    {
        info!("\n\nFourth case");
        let res = call_with_retry_condition(
            async || make_request(&fourth_qs).await,
            |e: &AppError| {
                if e.is_timeout() {
                    debug!("Request timed out. Retrying...");
                    true
                } else {
                    debug!("Request failed.");
                    false
                }
            },
        )
        .instrument(info_span!("retry_block_fourth")) // FIXME: Somehow this doesn't show up.
        .await;
        assert!(res.is_ok());
        let res = res.unwrap();
        info!("res: {:?}", res);
    }

    Ok(())
}

#[instrument(name = "make_request", skip_all, ret)]
async fn make_request(qs: &String) -> Result<String, AppError> {
    // Get the OpenTelemetry `Context` via the current `tracing::Span`.
    let cx = tracing::Span::current().context();

    // Initialize the request builder.
    let mut request_builder = reqwest::ClientBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .unwrap()
        .get(format!("http://localhost:8000/hello/world{}", qs));

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
    let res = request_builder.send().await.map_err(|e| {
        if e.is_timeout() {
            AppError::new("timeout".to_string())
        } else {
            AppError::new(e.to_string())
        }
    })?;

    // Parse the response
    let body = res.text().await.map_err(|e| AppError::new(e.to_string()))?;
    debug!("body: {}", body);

    // If the body contains the word error, return an error
    if body.contains("error") {
        return Err(AppError::new("error".to_string()));
    }

    Ok(body)
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

#[derive(Debug)]
struct AppError {
    message: String,
}

impl AppError {
    fn new(message: String) -> Self {
        Self { message }
    }

    pub fn is_timeout(&self) -> bool {
        self.message.contains("timeout")
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "AppError: {}", self.message)
    }
}
