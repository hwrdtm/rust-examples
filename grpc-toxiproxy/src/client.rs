use grpc_toxiproxy::{DEFAULT_SERVER_PORT, PROXIED_SERVER_PORT};
use hello_world::greeter_client::GreeterClient;
use hello_world::HelloRequest;

pub mod hello_world {
    tonic::include_proto!("helloworld");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = {
        #[cfg(feature = "proxy_grpc")]
        {
            PROXIED_SERVER_PORT
        }

        #[cfg(not(feature = "proxy_grpc"))]
        {
            DEFAULT_SERVER_PORT
        }
    };

    // Time the duration it takes to make a request to the server.
    let start = std::time::Instant::now();

    println!("Making request to server at port {}", port);
    let mut client = GreeterClient::connect(format!("http://127.0.0.1:{}", port))
        .await
        .expect("Failed to connect to server");

    let request = tonic::Request::new(HelloRequest {
        name: "Tonic".into(),
    });

    let response = client.say_hello(request).await?;

    println!("RESPONSE={:?}", response);
    println!("Request took {}ms", start.elapsed().as_millis());

    Ok(())
}
