use warp::Filter;

#[tokio::main]
async fn main() {
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

    warp::serve(hello).run(([127, 0, 0, 1], port)).await;
}
