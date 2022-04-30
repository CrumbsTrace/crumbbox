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
        .multipart(Form::new().text("relative_path", "").part("file", part))
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
        .multipart(Form::new().text("relative_path", ""))
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
        .multipart(Form::new().text("relative_path", "").part("file", part))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);

    let original_contents = get_file_contents("tests/fixtures/test.txt");
    let new_contents = get_file_contents(&format!("{}/{}", app.storage_path, file_name));
    assert_eq!(original_contents, new_contents);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
}

#[tokio::test]
async fn uploads_with_invalid_file_names_are_rejected() {
    let app = spawn_app().await;

    let too_long_file_name = "a".repeat(256);
    let too_long_file_name_double_byte = "あ".repeat(128);
    let invalid_file_names = vec!["/", &too_long_file_name, &too_long_file_name_double_byte];

    for file_name in invalid_file_names {
        let contents = get_file_contents("tests/fixtures/test.txt");
        let part = Part::bytes(contents).file_name(file_name.to_string());

        let client = reqwest::Client::new();
        let response = client
            .post(&format!("{}/upload", app.addr()))
            .multipart(Form::new().text("relative_path", "").part("file", part))
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            response.status(),
            reqwest::StatusCode::BAD_REQUEST,
            "Illegal file name: {}",
            file_name
        );
    }
}

#[tokio::test]
async fn allow_for_upload_of_files_with_spaces() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.txt");
    let file_name = "test with spaces.txt";
    let part = Part::bytes(contents).file_name(file_name.to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(Form::new().text("relative_path", "").part("file", part))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
}

#[tokio::test]
async fn allow_upload_of_image_files() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.jpg");
    let file_name = "test.png";
    let part = Part::bytes(contents).file_name(file_name.to_string());

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(Form::new().text("relative_path", "").part("file", part))
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
}

#[tokio::test]
async fn support_multiple_file_uploads_in_same_request() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.txt");
    let contents2 = get_file_contents("tests/fixtures/test.txt");
    let file_name = Uuid::new_v4().to_string();
    let file_name2 = Uuid::new_v4().to_string();
    let part = Part::bytes(contents).file_name(file_name.clone());
    let part2 = Part::bytes(contents2).file_name(file_name2.clone());

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(
            Form::new()
                .text("relative_path", "")
                .part("file", part)
                .part("file", part2),
        )
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name)).unwrap();
    std::fs::remove_file(format!("{}/{}", app.storage_path, file_name2)).unwrap();
}

#[tokio::test]
async fn upload_request_without_relative_path_is_rejected() {
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

    assert_eq!(response.status(), reqwest::StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn upload_request_without_multipart_form_is_rejected() {
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
async fn support_passing_a_file_path_to_use() {
    let app = spawn_app().await;
    let contents = get_file_contents("tests/fixtures/test.txt");
    let file_name = Uuid::new_v4().to_string();
    let part = Part::bytes(contents).file_name(file_name.clone());
    let relative_path = "directed_path_folder/";

    let client = reqwest::Client::new();
    let response = client
        .post(&format!("{}/upload", app.addr()))
        .multipart(
            Form::new()
                .text("relative_path", relative_path)
                .part("file", part),
        )
        .send()
        .await
        .expect("Failed to execute request");

    assert_eq!(response.status(), reqwest::StatusCode::OK);
    let final_path = format!("{}/{}/{}", app.storage_path, relative_path, file_name);
    std::fs::remove_file(final_path).unwrap();
}

fn get_file_contents(file_path: &str) -> Vec<u8> {
    let file = File::open(file_path).unwrap();
    let mut buf_reader = BufReader::new(file);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    contents
}
