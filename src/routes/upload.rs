use anyhow::Context;
use axum::{
    body::Bytes, extract::Multipart, http::StatusCode, response::IntoResponse, BoxError, Extension,
};
use futures::{Stream, TryStreamExt};
use std::io;
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::validators::validate_file_name;

#[derive(thiserror::Error, Debug)]
pub enum UploadError {
    #[error("{0}")]
    ValidationError(String),
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl IntoResponse for UploadError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            UploadError::UnexpectedError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            UploadError::ValidationError(_) => StatusCode::BAD_REQUEST,
        };

        tracing::error!("{}", self);
        (status, self.to_string()).into_response()
    }
}

#[tracing::instrument(name = "Upload multipart form", skip(multipart))]
pub async fn upload(
    mut multipart: Multipart,
    Extension(storage_path): Extension<String>,
) -> Result<(), UploadError> {
    tracing::info!("Received multipart form upload request");
    while let Some(field) = multipart
        .next_field()
        .await
        .context("Failed to read multipart field")?
    {
        let file_name = if let Some(file_name) = field.file_name() {
            file_name.to_owned()
        } else {
            continue;
        };

        validate_file_name(&file_name).map_err(UploadError::ValidationError)?;

        let file_path = format!("{}/{}", storage_path, file_name);
        tracing::info!("Saving file to: {}", file_path);
        stream_to_file(&file_path, field)
            .await
            .context("Failed to save file")?;
    }

    Ok(())
}

async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), io::Error>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    // Convert the stream into an `AsyncRead`.
    let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
    let body_reader = StreamReader::new(body_with_io_error);
    futures::pin_mut!(body_reader);

    // Create the file. `File` implements `AsyncWrite`.
    let mut file = BufWriter::new(File::create(path).await?);

    // Copy the body into the file.
    tokio::io::copy(&mut body_reader, &mut file).await?;

    Ok(())
}
