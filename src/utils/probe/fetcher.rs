use crate::{error_handler::ApiError, utils::probe::model::ResolvedMedia};
use reqwest::{header, Client, StatusCode};
use std::io::Write;
use tempfile::NamedTempFile;

pub const CHUNK_SIZE: u64 = 8 * 1024 * 1024;

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
impl ResolvedMedia {
    async fn fetch_range(&self, start: u64, len: u64) -> Result<Vec<u8>, ApiError> {
        let client = Client::new();
        let end = start
            .checked_add(len)
            .and_then(|v| v.checked_sub(1))
            .ok_or_else(|| ApiError::new(500, "Invalid range parameters"))?;

        let range = format!("bytes={}-{}", start, end);

        let response = client
            .get(&self.url)
            .headers(self.headers.clone())
            .header(header::RANGE, range)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to request file range: {e}")))?;

        if !response.status().is_success() && response.status() != StatusCode::PARTIAL_CONTENT {
            let status = response.status();
            return Err(ApiError::new(
                status.as_u16(),
                &format!("Failed to request file range: {status}"),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ApiError::new(500, &format!("Failed to read file bytes: {e}")))?;

        Ok(bytes.to_vec())
    }

    pub async fn fetch_head(&self) -> Result<Vec<u8>, ApiError> {
        self.fetch_range(0, CHUNK_SIZE).await
    }

    pub async fn fetch_from_offset(&self, offset: u64) -> Result<Vec<u8>, ApiError> {
        self.fetch_range(offset, CHUNK_SIZE).await
    }
}
