use std::{
    fs::File,
    io::{BufReader, Read},
};

use crate::helpers::spawn_app;
use reqwest::multipart::{Form, Part};
use uuid::Uuid;

#[tokio::test]
async fn file_upload_request_returns_200() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.txt");
    let file_name = Uuid::new_v4().to_string();
    let part = Part::bytes(contents).file_name(file_name.clone());

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(Form::new().part("file", part))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
}

#[tokio::test]
async fn file_upload_request_returns_400_when_no_file_is_provided() {
    let app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .post(&format!("{}/upload", app.addr()))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn uploaded_file_is_identical_to_original_file() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.txt");
    let file_name = Uuid::new_v4().to_string();
    let part = Part::bytes(contents).file_name(file_name.clone());

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(Form::new().part("file", part))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let original_contents = get_file_contents("tests/fixtures/test.txt");
    let new_contents = get_file_contents(&format!("{}/{}", app.storage_path, file_name));
    assert_eq!(original_contents, new_contents);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
}

fn get_file_contents(file_path: &str) -> Vec<u8> {
    let file = File::open(file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    contents
}
