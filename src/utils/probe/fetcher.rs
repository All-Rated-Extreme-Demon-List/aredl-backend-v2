use crate::{error_handler::ApiError, utils::probe::model::ResolvedMedia};
use reqwest::Client;
use std::io::Write;
use tempfile::NamedTempFile;

const CHUNK_SIZE: u64 = 4 * 1024 * 1024;

pub async fn save_to_temp_file(bytes: &[u8]) -> Result<std::path::PathBuf, ApiError> {
    let mut file = NamedTempFile::new()
        .map_err(|e| ApiError::new(500, &format!("Failed to create temp file: {e}")))?;

    file.write_all(bytes)
        .map_err(|e| ApiError::new(500, &format!("Failed to write temp file: {e}")))?;

    let path = file.path().to_path_buf();

    file.keep()
        .map_err(|e| ApiError::new(500, &format!("Failed to persist temp file: {e}")))?;

    Ok(path)
}

pub async fn fetch_media(media: &ResolvedMedia) -> Result<Vec<u8>, ApiError> {
    let client = Client::new();

    let response = client
        .get(&media.url)
        .headers(media.headers.clone())
        .header(
            reqwest::header::RANGE,
            format!("bytes=0-{}", CHUNK_SIZE - 1),
        )
        .send()
        .await
        .map_err(|e| ApiError::new(502, &format!("Failed to request file content: {e}")))?;

    if !response.status().is_success() && response.status() != reqwest::StatusCode::PARTIAL_CONTENT
    {
        return Err(ApiError::new(
            502,
            &format!("Failed to request file content: {}", response.status()),
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| ApiError::new(500, &format!("Failed to convert file bytes: {e}")))?;
    Ok(bytes.to_vec())
}
