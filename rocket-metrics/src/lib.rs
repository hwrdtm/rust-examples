use opentelemetry::{global, KeyValue};
use opentelemetry_semantic_conventions::resource::URL_PATH;
use rocket::{fairing::Fairing, Data, Request, Response};

pub struct MetricsFairings;

#[rocket::async_trait]
impl Fairing for MetricsFairings {
    fn info(&self) -> rocket::fairing::Info {
        rocket::fairing::Info {
            name: "Metrics Fairing",
            kind: rocket::fairing::Kind::Request | rocket::fairing::Kind::Response,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _: &mut Data<'_>) {
        // Get the SDK-Version header value.
        let sdk_version = req.headers().get_one("SDK-Version").unwrap_or("unknown");

        let meter = global::meter("http");
        let counter = meter
            .u64_counter("service.request")
            .with_description("Counter for HTTP requests.")
            .init();
        counter.add(
            1,
            &[
                KeyValue::new("method", req.method().as_str()),
                KeyValue::new(URL_PATH, req.uri().path().to_string()),
                KeyValue::new("sdk.version", sdk_version.to_owned()),
            ],
        );
    }

    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let meter = global::meter("http");
        let counter = meter
            .u64_counter("service.response")
            .with_description("Counter for HTTP responses.")
            .init();
        counter.add(
            1,
            &[
                KeyValue::new("method", req.method().as_str()),
                KeyValue::new("status", res.status().to_string()),
            ],
        );
    }
}
