use anyhow::Context;
use axum::{
    body::Bytes,
    extract::{multipart::Field, Multipart},
    http::StatusCode,
    response::IntoResponse,
    BoxError, Extension,
};
use futures::{Stream, TryStreamExt};
use std::{io, sync::Arc};
use tokio::{fs::File, io::BufWriter};
use tokio_util::io::StreamReader;

use crate::{domain::StorageDetails, validators::validate_file_name};

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

#[tracing::instrument(name = "Upload multipart form", skip(multipart, storage_details))]
pub async fn upload(
    mut multipart: Multipart,
    storage_details: Extension<Arc<StorageDetails>>,
) -> Result<(), UploadError> {
    if let Some(path_field) = get_multipart_field(&mut multipart).await? {
        if path_field.name().context("No field name")? != "relative_path" {
            return Err(UploadError::ValidationError(
                "Expected field name 'relative_path'".to_string(),
            ));
        };

        let relative_path = path_field.text().await.context("Failed to get text")?;
        let base_path = format!("{}/{}", storage_details.path, relative_path);
        let mut uploaded_files = false;

        while let Some(field) = get_multipart_field(&mut multipart).await? {
            uploaded_files = true;
            let file_name = field.file_name().context("Failed to get file name")?;
            validate_file_name(file_name).map_err(UploadError::ValidationError)?;

            let file_path = format!("{}/{}", base_path, file_name);
            stream_to_file(&file_path, field)
                .await
                .context("Failed to save file")?;
        }

        if !uploaded_files {
            return Err(UploadError::ValidationError(
                "No files were in the multipart form".to_string(),
            ));
        }
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

//This clippy lint is currently disabled here due to a bug https://github.com/rust-lang/rust-clippy/issues/5787
#[allow(clippy::needless_lifetimes)]
async fn get_multipart_field<'a>(
    multipart: &'a mut Multipart,
) -> Result<Option<Field<'a>>, UploadError> {
    match multipart.next_field().await {
        Ok(field) => Ok(field),
        Err(err) => Err(UploadError::UnexpectedError(err.into())),
    }
}
