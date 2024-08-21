#[macro_use] extern crate rocket;

use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::primitives::ByteStream;
use rocket::http::Status;
use serde_json::{json, Value};
use dotenv::dotenv;
use rocket::fs::TempFile;
use rocket::form::Form;
use rocket::response::status::Custom;
use aws_sdk_s3 as s3;

#[derive(Debug, FromForm)]
struct UploadForm<'r> {
    file: TempFile<'r>,
}

#[post("/upload", data = "<form>")]
async fn upload(form: Form<UploadForm<'_>>) -> Result<Value, Custom<Value>> {
    let temp_file = &form.file;
    let file_name = temp_file.name().unwrap();
    let byte_stream = std::fs::read(temp_file.path().unwrap()).unwrap();
    let body = ByteStream::from(byte_stream);

    dotenv().ok();

    let aws_access_key_id = std::env::var("AWS_ACCESS_KEY_ID").expect("AWS_ACCESS_KEY_ID must be set.");
    let aws_secret_access_key = std::env::var("AWS_SECRET_ACCESS_KEY").expect("AWS_SECRET_ACCESS_KEY must be set.");
    let aws_region = std::env::var("AWS_REGION").expect("AWS_REGION must be set.");
    let s3_bucket = std::env::var("S3_BUCKET").expect("AWS_BUCKET must be set.");

    let creds = Credentials::new(aws_access_key_id, aws_secret_access_key, None, None, "ag-provides");
    let my_config = aws_config::defaults(BehaviorVersion::latest())
        .test_credentials()
        .region(Region::new(aws_region))
        .credentials_provider(creds)
        .load()
        .await;

    let s3_client = s3::Client::new(&my_config);

     let put_object =  s3_client.put_object()
        .bucket(s3_bucket)
        .key(file_name)
        .body(body)
        .send()
        .await;

    if put_object.is_ok() {
        Ok(json!({
            "status": "success",
            "message": "File uploaded successfully"
        }))
    } else {
        Err(Custom(Status::InternalServerError, json!({
            "status": "error",
            "message": format!("Failed to upload file:")
        })))
    }
}

#[rocket::main]
async fn main() {
    let _ = rocket::build()
        .configure(rocket::Config::figment().merge(("port", 9797)))
        .mount("/", routes![upload])
        .launch()
        .await;
}