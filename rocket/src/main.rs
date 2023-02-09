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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct AdminAuthSig {
    pub auth_sig: JsonAuthSig,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct JsonAuthSig {
    pub sig: String,
    pub derived_via: String,
    pub signed_message: String,
    pub address: String,
    pub capabilities: Option<Vec<JsonAuthSig>>,
    pub algo: Option<String>,
}

/// The AdminAuthSig request guard is used to check for the existence of the x-auth-sig header.
/// If it is not present, the request is rejected as unauthorized.
/// If it is present, it is decoded from base64, deserialized as JSON and returned as a JsonAuthSig struct.
#[rocket::async_trait]
impl<'r> FromRequest<'r> for AdminAuthSig {
    type Error = Value;

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        println!("request: {:?}", request);

        let auth_sig = request.headers().get_one("x-auth-sig");
        if auth_sig.is_none() {
            return Outcome::Failure((
                Status::Unauthorized,
                json!({"error": "Missing x-auth-sig header"}),
            ));
        }

        // Decode base64.
        let decoded_auth_sig = base64::decode(auth_sig.unwrap());
        if let Err(e) = decoded_auth_sig {
            return Outcome::Failure((
                Status::Unauthorized,
                json!({"error": "Unable to decode base64", "reason": e.to_string()}),
            ));
        }

        // Deserialize JSON.
        let deserialized_auth_sig =
            serde_json::from_slice::<JsonAuthSig>(&decoded_auth_sig.unwrap());
        if let Err(e) = deserialized_auth_sig {
            return Outcome::Failure((
                Status::Unauthorized,
                json!({"error": "Unable to deserialize JSON", "reason": e.to_string()}),
            ));
        }

        return Outcome::Success(AdminAuthSig {
            auth_sig: deserialized_auth_sig.unwrap(),
        });
    }
}

#[post("/web/admin/set_key_backup", format = "plain", data = "<file>")]
async fn upload(auth_sig: AdminAuthSig, mut file: TempFile<'_>) -> status::Custom<Value> {
    println!("auth_sig: {:?}", auth_sig.auth_sig);

    // Stream the file to disk.
    let zipped_file_path = "/var/tmp/test_backup.tar.gz";
    let persist_tempfile_res = file.persist_to(zipped_file_path).await;
    if let Err(e) = persist_tempfile_res {
        return status::Custom(
            Status::InternalServerError,
            json!({
                "error": "Unable to persist tempfile",
                "reason": e.to_string(),
            }),
        );
    }
    println!("File saved to: {}", zipped_file_path);

    // Unzip the file, which should replace the existing key material.
    let unzip_result = unzip_and_delete_file(zipped_file_path).await;
    if let Err(e) = unzip_result {
        return status::Custom(
            Status::InternalServerError,
            json!({
                "error": "Unable to unzip keys",
                "reason": e.to_string(),
            }),
        );
    }

    println!("Keys unzipped");

    return status::Custom(
        Status::Ok,
        json!({
            "success": "true",
        }),
    );
}

#[get("/")]
fn index() -> &'static str {
    // Log request.
    println!("Received request");

    "
    USAGE

      POST /

          accepts raw data in the body of the request and responds with a URL of
          a page containing the body's content

      GET /<id>

          retrieves the content for the paste with id `<id>`
    "
}

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
        .mount("/", routes![index, upload])
        .attach(cors.clone())
}

pub async fn unzip_and_delete_file(zipped_file_path: &str) -> Result<()> {
    // unzip
    Command::new("tar")
        .arg("-xzvf")
        .arg(zipped_file_path)
        .output()
        .await?;

    // delete the zipped file
    Command::new("rm").arg(zipped_file_path).output().await?;

    Ok(())
}
