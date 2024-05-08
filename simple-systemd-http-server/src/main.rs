use std::{convert::Infallible, time::Duration};

use serde_derive::{Deserialize, Serialize};

use warp::Filter;

#[tokio::main]
async fn main() {
    env_logger::init();

    // Read SERVER_PORT environment variable
    let port = std::env::var("SERVER_PORT")
        .expect("SERVER_PORT must be set")
        .parse::<u16>()
        .expect("Invalid SERVER_PORT");

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String).map(move |name| {
        println!("[{}] Received request for name: {}", port, name);
        format!("[{}] Hello, {}!", port, name)
    });

    // POST /hello => 200 OK
    let forward = warp::post()
        .and(warp::path("forward"))
        .and(warp::body::json())
        .and_then(sleepy);

    println!("[{}] Starting server...", port);

    let routes = hello.or(forward);
    warp::serve(routes).run(([127, 0, 0, 1], port)).await;
}

async fn sleepy(mut fd: ForwardDetails) -> Result<impl warp::Reply, Infallible> {
    println!("Forwarding to: {}", fd.forward_url);

    // Make a request to the forward_url
    let response = match reqwest::Client::new().get(&fd.forward_url).send().await {
        Ok(response) => response,
        Err(e) => {
            return Ok(format!("Error: {}", e));
        }
    };

    let body = response.text().await.unwrap();

    Ok(format!("Response from {}: {}", fd.forward_url, body))
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ForwardDetails {
    pub forward_url: String,
}
