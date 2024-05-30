use opentelemetry_proto::tonic::collector::metrics::v1::metrics_service_server::{
    MetricsService, MetricsServiceServer,
};
use opentelemetry_proto::tonic::collector::metrics::v1::{
    ExportMetricsServiceRequest, ExportMetricsServiceResponse,
};
use simple_observability_pipeline::DEFAULT_SOCK;
use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};

#[derive(Debug, Default)]
pub struct OTELProxyServer {}

#[tonic::async_trait]
impl MetricsService for OTELProxyServer {
    async fn export(
        &self,
        request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        println!("Got a request: {:?}", request);

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
        .add_service(MetricsServiceServer::new(server))
        .serve_with_incoming(uds_stream)
        .await?;

    Ok(())
}
