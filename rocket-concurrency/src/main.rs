#[macro_use]
extern crate rocket;

use anyhow::Result;
use rocket::fs::TempFile;
use rocket::http::{Method, Status};
use rocket::request::{self, FromRequest, Outcome, Request};
use rocket::response::status;
use rocket::serde::json::{serde_json::json, Value};
use rocket_cors::{AllowedOrigins, CorsOptions};
use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[launch]
fn rocket() -> _ {
    let cors = CorsOptions::default()
        .allowed_origins(AllowedOrigins::all())
        .allowed_methods(
            vec![Method::Get, Method::Post, Method::Patch]
                .into_iter()
                .map(From::from)
                .collect(),
        )
        .allow_credentials(true)
        .to_cors()
        .expect("CORS failed to build");

    rocket::build()
        .mount("/", routes![tokio_sleep, core_sleep])
        .attach(cors.clone())
}

#[get("/tokio-sleep")]
async fn tokio_sleep() -> &'static str {
    // Log request.
    println!("[tokio_sleep] Received request");

    // Sleep for 10 seconds.
    tokio::time::sleep(std::time::Duration::from_secs(10)).await;

    "
    ret from tokio_sleep!
    "
}

#[get("/core-sleep")]
async fn core_sleep() -> &'static str {
    // Log request.
    println!("[core_sleep] Received request");

    // Sleep for 10 seconds.
    std::thread::sleep(std::time::Duration::from_secs(10));

    "
    ret from core_sleep!
    "
}
