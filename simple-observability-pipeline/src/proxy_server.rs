use std::io::Write;

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
use simple_observability_pipeline::DEFAULT_SOCK;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};

const LOCAL_COMBINED_OUT: &str = "./otel_combined.log";
const LOCAL_LOG_FILE_OUT: &str = "./otel_logs.log";
const LOCAL_METRICS_FILE_OUT: &str = "./otel_metrics.log";
const LOCAL_TRACES_FILE_OUT: &str = "./otel_traces.log";

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = OTELProxyServer::default();

    // If the socket already exists, remove it
    if std::fs::metadata(DEFAULT_SOCK).is_ok() {
        std::fs::remove_file(DEFAULT_SOCK)?;
    }

    let uds = UnixListener::bind(DEFAULT_SOCK)?;
    let uds_stream = UnixListenerStream::new(uds);
    println!("Listening on {:?}", uds_stream);

    Server::builder()
        .add_service(TraceServiceServer::new(server.clone()))
        .add_service(MetricsServiceServer::new(server.clone()))
        .add_service(LogsServiceServer::new(server))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
