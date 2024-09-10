use std::time::Duration;

use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{global, metrics::MetricsError, KeyValue};
use opentelemetry_otlp::TonicExporterBuilder;
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        SdkMeterProvider,
    },
    Resource,
};
use rocket_metrics::MetricsFairings;

#[macro_use]
extern crate rocket;

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "rocket-server",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::URL_DOMAIN,
            "localhost",
        ),
    ])
});

mod routes {
    use rocket::serde::json::Json;
    use serde::Deserialize;

    #[get("/hello/<name>?<caps>")]
    pub fn hello(name: &str, caps: Option<bool>) -> String {
        let name = caps
            .unwrap_or_default()
            .then(|| name.to_uppercase())
            .unwrap_or_else(|| name.to_string());
        format!("Hello, {}!", name)
    }

    #[derive(Deserialize)]
    pub struct Person {
        age: u8,
    }

    #[post("/hello/<name>?<caps>", format = "json", data = "<person>")]
    pub fn hello_post(name: String, person: Json<Person>, caps: Option<bool>) -> String {
        let name = caps
            .unwrap_or_default()
            .then(|| name.to_uppercase())
            .unwrap_or_else(|| name.to_string());
        format!("Hello, {} year old named {}!", person.age, name)
    }
}

fn main() -> Result<(), rocket::Error> {
    let logging_rt = tokio::runtime::Runtime::new().expect("Failed to create logging runtime");
    logging_rt
        .block_on(init_observability())
        .expect("Failed to initialize observability");

    let main_rt = tokio::runtime::Runtime::new().expect("Failed to create main runtime");
    let _rocket = main_rt.block_on(async {
        rocket::build()
            .attach(MetricsFairings)
            .mount("/", routes![routes::hello, routes::hello_post])
            .launch()
            .await
    })?;

    Ok(())
}

async fn init_observability() -> Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::new_exporter().tonic();

    // Initialize the metrics pipeline
    let meter_provider = init_metrics(exporter, RESOURCE.clone())?;

    // Set globals
    global::set_meter_provider(meter_provider.clone());

    Ok(meter_provider)
}

fn init_metrics(
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
