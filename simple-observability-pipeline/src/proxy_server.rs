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

const LOCAL_FILE_OUT: &str = "./otel_proxy_server.log";

#[derive(Debug, Default, Clone)]
pub struct OTELProxyServer {}

impl OTELProxyServer {
    fn write_otel_request_to_file(&self, request: &str) -> Result<(), Status> {
        // Write the request as JSON appended to a log file
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(LOCAL_FILE_OUT)
            .map_err(|e| Status::internal(format!("Failed to open log file: {}", e)))?;
        writeln!(file, "{}", request)
            .map_err(|e| Status::internal(format!("Failed to write to log file: {}", e)))?;

        Ok(())
    }
}

#[tonic::async_trait]
impl TraceService for OTELProxyServer {
    async fn export(
        &self,
        request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        println!("Got a trace request: {:?}", request);

        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(&request)?;

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
        println!("Got a log request: {:?}", request);

        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(&request)?;

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
        println!("Got a metrics request: {:?}", request);

        let request = serde_json::to_string(&request.into_inner())
            .map_err(|e| Status::internal(format!("Failed to serialize request: {}", e)))?;
        self.write_otel_request_to_file(&request)?;

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

    // If the log file already exists, remove it
    if std::fs::metadata(LOCAL_FILE_OUT).is_ok() {
        std::fs::remove_file(LOCAL_FILE_OUT)?;
    }
    // Create the log file
    std::fs::File::create(LOCAL_FILE_OUT)?;

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
