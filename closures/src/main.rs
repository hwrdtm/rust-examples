// use std::{sync::Arc, net::SocketAddr};

// use hyper::{service::{make_service_fn, service_fn}, Server};

// #[tokio::main]
// async fn main() {
//     let cfg = Arc::new(Config::new());

//     let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

//     let router = Arc::new(RequestRouter {
//         config: cfg,
//     });

//     let make_service = make_service_fn(move |conn| {
//         let router_arc = Arc::clone(&router);
//         let service = service_fn(move |req| router_arc.route_request(req));
//         async move { Ok::<_, hyper::Error>(service) }
//     });

//     // Then bind and serve...
//     let server = Server::bind(&addr).serve(make_service);

//     // And run forever...
//     if let Err(e) = server.await {
//         eprintln!("server error: {}", e);
//     }
// }

// pub struct RequestRouter {
//     pub config: Arc<Config>,
// }

// impl RequestRouter {
//     pub async fn route_request(&self, req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
//         let mut response = Response::new(Body::empty());

//         match (req.method(), req.uri().path()) {
//             (&Method::GET, "/") => {
//                 *response.body_mut() = Body::from("Hello, World!");
//             }
//             (&Method::GET, "/config") => {
//                 let config = self.config.clone();
//                 let config_json = serde_json::to_string(&config).unwrap();
//                 *response.body_mut() = Body::from(config_json);
//             }
//             _ => {
//                 *response.status_mut() = StatusCode::NOT_FOUND;
//             }
//         }

//         Ok(response)
//     }
// }

// pub struct Config {
//     pub key: String,
// }

// impl Config {
//     pub fn new() -> Self {
//         Self {
//             key: "value".to_string(),
//         }
//     }
// }

fn main() {
    println!("Hello, world!");
}
