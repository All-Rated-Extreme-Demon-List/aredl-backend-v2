use std::{fs::remove_file, process::Stdio, sync::Arc};

use regex::Regex;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use tokio::{join, process::Command};
use utoipa::ToSchema;

use crate::{
    app_data::drive::DriveState,
    error_handler::ApiError,
    utils::probe::fetcher::{fetch_media, save_to_temp_file},
};

#[derive(Debug, Clone, Copy, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MediaProvider {
    GoogleDrive,
}

#[derive(Debug, Clone)]
pub struct ProviderMatch {
    pub provider: MediaProvider,
    pub id: String,
}

#[derive(Debug, Clone)]
pub struct ResolvedMedia {
    pub url: String,
    pub headers: HeaderMap,
}
#[derive(Deserialize, ToSchema)]
pub struct ProbeRequest {
    pub url: String,
}

#[derive(Serialize, ToSchema)]
pub struct ProbeResponse {
    pub probed_url: String,
    pub exif: Option<serde_json::Value>,
    pub ffprobe: Option<serde_json::Value>,
}

pub fn detect_provider(input: &str) -> Option<ProviderMatch> {
    const DRIVE_PATTERNS: [&str; 3] = [
        r#"https?://drive\.google\.com/file/d/([\w-]+)"#,
        r#"https?://drive\.google\.com/open\?id=([\w-]+)"#,
        r#"https?://drive\.google\.com/uc\?(?:[^#]*?)id=([\w-]+)"#,
    ];

    for pat in DRIVE_PATTERNS {
        let re = Regex::new(pat).unwrap();
        if let Some(caps) = re.captures(input) {
            let file_id = caps[1].to_string();
            return Some(ProviderMatch {
                provider: MediaProvider::GoogleDrive,
                id: file_id,
            });
        }
    }

    None
}

async fn run_ffprobe(path: &str) -> Result<serde_json::Value, ApiError> {
    let mut command = Command::new("/usr/local/bin/ffprobe");
    command
        .arg("-v")
        .arg("error")
        .arg("-hide_banner")
        .arg("-probesize")
        .arg("1048576")
        .arg("-analyzeduration")
        .arg("1000000")
        .arg("-show_format")
        .arg("-show_streams")
        .arg("-show_entries")
        .arg("format_tags:stream_tags")
        .arg("-print_format")
        .arg("json")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| ApiError::new(500, &format!("Failed to run ffprobe: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::new(
            422,
            &format!("Failed to run ffprobe: {stderr}"),
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| ApiError::new(500, &format!("Failed to decode ffprobe output: {e}")))?;
    let value: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| ApiError::new(500, &format!("Failed to parse ffprobe output: {e}")))?;

    Ok(value)
}

async fn run_exiftool(path: &str) -> Result<serde_json::Value, ApiError> {
    let mut command = Command::new("/usr/bin/exiftool");
    command
        .arg("-j")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = command
        .output()
        .await
        .map_err(|e| ApiError::new(500, &format!("Failed to run exiftool: {e}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ApiError::new(
            422,
            &format!("Failed to run exiftool: {stderr}"),
        ));
    }

    let stdout = String::from_utf8(output.stdout)
        .map_err(|e| ApiError::new(500, &format!("Failed to decode exiftool output: {e}")))?;

    let value: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| ApiError::new(500, &format!("Failed to parse exiftool output: {e}")))?;

    Ok(value)
}

pub async fn probe_url(media: &ResolvedMedia) -> Result<ProbeResponse, ApiError> {
    let file_bytes = fetch_media(media).await?;

    let file_path = save_to_temp_file(&file_bytes)
        .await?
        .to_str()
        .unwrap()
        .to_string();

    let (ffprobe_result, exif_result) =
        join!(async { run_ffprobe(&file_path).await.ok() }, async {
            run_exiftool(&file_path).await.ok()
        },);

    let _ = remove_file(&file_path);

    Ok(ProbeResponse {
        probed_url: media.url.clone(),
        exif: exif_result,
        ffprobe: ffprobe_result,
    })
}

pub async fn resolve_media(
    provider_match: &ProviderMatch,
    drive_state: &Option<Arc<DriveState>>,
) -> Result<ResolvedMedia, ApiError> {
    match provider_match.provider {
        MediaProvider::GoogleDrive => {
            let drive_state = drive_state
                .as_ref()
                .ok_or_else(|| ApiError::new(500, "Google Drive support isn't available"))?;

            let token = drive_state
                .get_access_token()
                .await
                .map_err(|e| ApiError::new(502, &format!("Failed to acquire Drive token: {e}")))?;

            let url = format!(
                "https://www.googleapis.com/drive/v3/files/{}?alt=media&supportsAllDrives=true",
                provider_match.id
            );

            let mut headers = HeaderMap::new();
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
            );

            Ok(ResolvedMedia { url, headers })
        }
    }
}
