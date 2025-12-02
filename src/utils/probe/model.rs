use std::{fs::remove_file, process::Stdio, sync::Arc};

use regex::Regex;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};
use tokio::{join, process::Command};
use utoipa::ToSchema;

use crate::{
    app_data::drive::DriveState, error_handler::ApiError, utils::probe::fetcher::save_to_temp_file,
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
pub struct ProbeField {
    pub data: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Serialize, ToSchema)]
pub struct ProbeResponse {
    pub probed_url: String,
    pub exif: ProbeField,
    pub ffprobe: ProbeField,
}

impl ProbeField {
    pub fn new() -> Self {
        Self {
            data: None,
            error: None,
        }
    }

    pub fn set_ok(&mut self, v: JsonValue) {
        self.data = Some(v);
        self.error = None;
    }

    pub fn set_err<S: Into<String>>(&mut self, msg: S) {
        self.error = Some(msg.into());
    }
}

impl ProviderMatch {
    pub fn available_providers() -> Vec<(MediaProvider, Vec<Regex>)> {
        vec![(
            MediaProvider::GoogleDrive,
            vec![
                Regex::new(r"https?://drive\.google\.com/file/d/([\w-]+)").unwrap(),
                Regex::new(r"https?://drive\.google\.com/open\?id=([\w-]+)").unwrap(),
                Regex::new(r"https?://drive\.google\.com/uc\?(?:[^#]*?)id=([\w-]+)").unwrap(),
            ],
        )]
    }

    pub fn from_url(url: &str) -> Option<Self> {
        for (provider, regexes) in Self::available_providers() {
            for re in regexes {
                if let Some(caps) = re.captures(url) {
                    if let Some(m) = caps.get(1) {
                        return Some(ProviderMatch {
                            provider,
                            id: m.as_str().to_string(),
                        });
                    }
                }
            }
        }
        None
    }

    pub async fn resolve(
        self: &Self,
        drive_state: &Option<Arc<DriveState>>,
    ) -> Result<ResolvedMedia, ApiError> {
        match self.provider {
            MediaProvider::GoogleDrive => {
                let drive_state = drive_state
                    .as_ref()
                    .ok_or_else(|| ApiError::new(500, "Google Drive support isn't available"))?;

                let token = drive_state.get_access_token().await.map_err(|e| {
                    ApiError::new(502, &format!("Failed to acquire Drive token: {e}"))
                })?;

                let url = format!(
                    "https://www.googleapis.com/drive/v3/files/{}?alt=media&supportsAllDrives=true",
                    self.id
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
}

struct ExifInfo<'a> {
    root: &'a JsonMap<String, JsonValue>,
}

impl<'a> ExifInfo<'a> {
    fn from_value(v: &'a JsonValue) -> Option<Self> {
        let object = match v {
            JsonValue::Array(array) if !array.is_empty() => array[0].as_object()?,
            JsonValue::Object(object) => object,
            _ => return None,
        };

        Some(ExifInfo { root: object })
    }

    fn get_str(&self, key: &str) -> Option<&str> {
        self.root.get(key)?.as_str()
    }

    fn get_u64(&self, key: &str) -> Option<u64> {
        self.root.get(key)?.as_u64()
    }

    fn is_mp4_like(&self) -> bool {
        let file_type = self.get_str("FileType");
        let mime = self.get_str("MIMEType");
        let ext = self.get_str("FileTypeExtension");

        file_type.map_or(false, |s| {
            ["mp4", "mov"].iter().any(|c| s.eq_ignore_ascii_case(c))
        }) || mime.map_or(false, |s| {
            let s = s.to_ascii_lowercase();
            s.contains("mp4") || s.contains("quicktime")
        }) || ext.map_or(false, |s| {
            ["mp4", "mov"].iter().any(|c| s.eq_ignore_ascii_case(c))
        })
    }

    fn moov_offset(&self) -> Option<u64> {
        let mdat_offset = self.get_u64("MediaDataOffset")?;
        let mdat_size = self.get_u64("MediaDataSize")?;
        Some(mdat_offset + mdat_size)
    }
}

impl ResolvedMedia {
    async fn get_ffprobe(path: &str) -> Result<JsonValue, ApiError> {
        let mut command = Command::new("/usr/local/bin/ffprobe");
        command
            .arg("-v")
            .arg("error")
            .arg("-hide_banner")
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

        let value: JsonValue = serde_json::from_str(&stdout)
            .map_err(|e| ApiError::new(500, &format!("Failed to parse ffprobe output: {e}")))?;

        Ok(value)
    }

    async fn get_exiftool(path: &str) -> Result<JsonValue, ApiError> {
        let mut command = Command::new("/usr/bin/exiftool");
        command
            .arg("-api")
            .arg("LargeFileSupport=1")
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

        let value: JsonValue = serde_json::from_str(&stdout)
            .map_err(|e| ApiError::new(500, &format!("Failed to parse exiftool output: {e}")))?;

        Ok(value)
    }

    pub async fn probe(&self) -> Result<ProbeResponse, ApiError> {
        let mut exif_field = ProbeField::new();
        let mut ffprobe_field = ProbeField::new();

        let head_bytes = self.fetch_head().await?;
        let head_path = save_to_temp_file(&head_bytes)
            .await?
            .to_str()
            .unwrap()
            .to_string();

        let (ffprobe_head_res, exif_head_res) =
            join!(async { Self::get_ffprobe(&head_path).await }, async {
                Self::get_exiftool(&head_path).await
            },);

        match ffprobe_head_res {
            Ok(v) => ffprobe_field.set_ok(v),
            Err(e) => ffprobe_field.set_err(e.error_message),
        }

        match exif_head_res {
            Ok(v) => exif_field.set_ok(v),
            Err(e) => exif_field.set_err(e.error_message),
        }

        let exif_value = exif_field.data.as_ref();
        let exif_info = exif_value.and_then(ExifInfo::from_value);

        // if ffprobe fails, and it's an mp4, try looking for the moov right after the mdat
        let try_mp4_moov = exif_info.as_ref().map_or(false, |info| {
            ffprobe_field.data.is_none() && info.is_mp4_like() && info.moov_offset().is_some()
        });

        if try_mp4_moov {
            if let Some(info) = exif_info {
                if let Some(moov_offset) = info.moov_offset() {
                    match self.fetch_from_offset(moov_offset).await {
                        Err(_e) => {}
                        Ok(moov_bytes) => {
                            let moov_path = save_to_temp_file(&moov_bytes)
                                .await?
                                .to_str()
                                .unwrap()
                                .to_string();

                            let (ffprobe_moov_res, exif_moov_res) =
                                join!(async { Self::get_ffprobe(&moov_path).await }, async {
                                    Self::get_exiftool(&moov_path).await
                                },);

                            match ffprobe_moov_res {
                                Ok(v) => {
                                    ffprobe_field.set_ok(v);
                                }
                                Err(e) => {
                                    ffprobe_field.set_err(e.error_message);
                                }
                            }
                            match exif_moov_res {
                                Ok(v) => {
                                    exif_field.set_ok(v);
                                }
                                Err(e) => {
                                    exif_field.set_err(e.error_message);
                                }
                            }
                            let _ = remove_file(&moov_path);
                        }
                    }
                }
            }
        }

        let _ = remove_file(&head_path);

        Ok(ProbeResponse {
            probed_url: self.url.clone(),
            exif: exif_field,
            ffprobe: ffprobe_field,
        })
    }
}
