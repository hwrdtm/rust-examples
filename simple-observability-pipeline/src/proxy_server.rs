use std::io::Write;
use std::time::Instant;

use anyhow::Result;
use once_cell::sync::Lazy;
use opentelemetry::{global, KeyValue};
use opentelemetry_proto::tonic::collector::logs::v1::logs_service_server::{
    LogsService, LogsServiceServer,
};
use opentelemetry_proto::tonic::collector::logs::v1::{
    ExportLogsServiceRequest, ExportLogsServiceResponse,
};
use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use opentelemetry_proto::tonic::collector::trace::v1::trace_service_server::{
    TraceService, TraceServiceServer,
};
use opentelemetry_proto::tonic::collector::trace::v1::{
    ExportTraceServiceRequest, ExportTraceServiceResponse,
};
use opentelemetry_sdk::logs::LoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::URL_PATH;
use simple_observability_pipeline::{create_providers, DEFAULT_SOCK};
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::async_trait;
use tonic::body::BoxBody;
use tonic::{transport::Server, Request, Response, Status};
use tonic_middleware::{Middleware, MiddlewareLayer, ServiceBound};
use tracing::error;
use tracing_subscriber::util::SubscriberInitExt;

const LOCAL_COMBINED_OUT: &str = "./otel_combined.log";
const LOCAL_LOG_FILE_OUT: &str = "./otel_logs.log";
const LOCAL_METRICS_FILE_OUT: &str = "./otel_metrics.log";
const LOCAL_TRACES_FILE_OUT: &str = "./otel_traces.log";

static RESOURCE: Lazy<Resource> = Lazy::new(|| {
    Resource::new(vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            "basic-otlp-server",
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::URL_DOMAIN,
            "localhost",
        ),
    ])
});

#[derive(Debug, Default, Clone)]
pub struct OTELProxyServer {}

impl OTELProxyServer {
    fn write_otel_request_to_file(&self, file_path: &str, request: &str) -> Result<(), Status> {
        // Write the request as JSON appended to a log file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(file_path)
            .map_err(|e| Status::internal(format!("Failed to open log file: {}", e)))?;
        writeln!(file, "{}", request)
            .map_err(|e| Status::internal(format!("Failed to write to log file: {}", e)))?;

        // Write the request as JSON to a combined log file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(LOCAL_COMBINED_OUT)
            .map_err(|e| Status::internal(format!("Failed to open combined log file: {}", e)))?;
        writeln!(file, "{}", request).map_err(|e| {
            Status::internal(format!("Failed to write to combined log file: {}", e))
        })?;

        Ok(())
    }
}

#[tonic::async_trait]
impl TraceService for OTELProxyServer {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(LOCAL_TRACES_FILE_OUT, &request)?;

        let reply = ExportTraceServiceResponse {
            partial_success: None,
        };

        Ok(Response::new(reply))
    }
}

#[tonic::async_trait]
impl LogsService for OTELProxyServer {
    async fn export(
        &self,
        request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(LOCAL_LOG_FILE_OUT, &request)?;

        let reply = ExportLogsServiceResponse {
            partial_success: None,
        };

        Ok(Response::new(reply))
    }
}

#[tonic::async_trait]
impl MetricsService for OTELProxyServer {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(LOCAL_METRICS_FILE_OUT, &request)?;

        let reply = ExportMetricsServiceResponse {
            partial_success: None,
        };

        Ok(Response::new(reply))
    }
}

struct Origin {
    service_name: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = OTELProxyServer::default();

    // If the socket already exists, remove it
    if std::fs::metadata(DEFAULT_SOCK).is_ok() {
        std::fs::remove_file(DEFAULT_SOCK)?;
    }

    let otel_service_rt =
        tokio::runtime::Runtime::new().expect("failed to create otel service runtime");

    let jh = otel_service_rt.spawn(async move {
        let uds = UnixListener::bind(DEFAULT_SOCK).expect("failed to bind to UDS socket");
        let uds_stream = UnixListenerStream::new(uds);
        println!("Listening on {:?}", uds_stream);

        Server::builder()
            .layer(MiddlewareLayer::new(MetricsMiddleware))
            .add_service(TraceServiceServer::new(server.clone()))
            .add_service(MetricsServiceServer::new(server.clone()))
            .add_service(LogsServiceServer::new(server))
            .serve_with_incoming(uds_stream)
            .await
            .expect("failed to serve")
    });

    // Init observability - this MUST occur after we have spawned the tasks to bring up the GRPC server.
    let observability_rt =
        tokio::runtime::Runtime::new().expect("failed to create Observability Runtime");
    let observability_providers = observability_rt.block_on(async {
        let observability_providers = init_observability().await?;
        Ok::<ObservabilityProviders, anyhow::Error>(observability_providers)
    })?;

    otel_service_rt.block_on(async {
        jh.await.expect("failed to join otel service runtime");
    });

    observability_providers.shutdown();

    Ok(())
}

use tonic::codegen::http::Request as HttpRequest; // Use this instead of tonic::Request in Middleware!
use tonic::codegen::http::Response as HttpResponse; // Use this instead of tonic::Response in Middleware!

#[derive(Default, Clone)]
pub struct MetricsMiddleware;

#[async_trait]
impl<S> Middleware<S> for MetricsMiddleware
where
    S: ServiceBound,
    S::Future: Send,
{
    async fn call(
        &self,
        req: HttpRequest<BoxBody>,
        mut service: S,
    ) -> Result<HttpResponse<BoxBody>, S::Error> {
        // If the request originated from this service itself, we don't want to track it.
        let req_headers = req.headers();
        let should_skip_metrics = match req_headers.get("x-origin") {
            Some(origin) => origin != "proxy-server",
            None => true,
        };
        if should_skip_metrics {
            return service.call(req).await;
        }

        // We should measure the duration of this request.
        let req_path = req.uri().path().to_owned();
        let start_time = Instant::now();
        let result = service.call(req).await;
        let elapsed_time = start_time.elapsed();

        // Send metrics events
        let meter = global::meter(format!("request"));
        meter
            .u64_counter("latency")
            .with_description("latency of requests")
            .with_unit("microseconds")
            .init()
            .add(
                elapsed_time.as_micros() as u64,
                &[KeyValue::new(URL_PATH, req_path)],
            );

        result
    }
}

async fn init_observability() -> Result<ObservabilityProviders> {
    let (tracing_provider, metrics_provider, subscriber, logger_provider) =
        create_providers(RESOURCE.clone(), true).await?;

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
