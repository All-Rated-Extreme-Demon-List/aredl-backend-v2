use serde::Deserialize;

use crate::{error_handler::ApiError, list::submissions::sync::RecordToSync, users::User};

#[derive(Debug, Deserialize)]
pub struct PemonlistPlayer {
    records: Vec<PemonlistRecord>,
}

#[derive(Debug, Deserialize)]
struct PemonlistRecord {
    formatted_time: String,
    level: PemonlistLevelInfo,
    mobile: bool,
    video_id: String,
}

#[derive(Debug, Deserialize)]
struct PemonlistLevelInfo {
    level_id: i32,
}

#[derive(Debug, Deserialize)]
struct PemonlistError {
    code: String,
    error: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PemonlistResponse {
    Err(PemonlistError),
    Ok(PemonlistPlayer),
}

pub struct Pemonlist;

impl Pemonlist {
    pub fn sync(user: User) -> Result<Vec<RecordToSync>, ApiError> {
        let Some(player_discord_id) = user.discord_id else {
            return Err(ApiError::UnprocessableEntity(
                "Given user does not have a discord id",
            ));
        };

        let client = reqwest::blocking::Client::new();
        let base_url = std::env::var("PEMONLIST_API_URL")
            .unwrap_or_else(|_| "https://pemonlist.com/api/player".to_owned());
        let url = format!("{}/{}", base_url.trim_end_matches('/'), player_discord_id);
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| ApiError::BadGateway(e.to_string()))?;

        let pemonlist_response: PemonlistResponse = resp.json().map_err(|e| {
            ApiError::BadGateway(format!("Failed to parse data received from pemonlist: {e}"))
        })?;

        let pemonlist_data = match pemonlist_response {
            PemonlistResponse::Err(err) if err.error && err.code == "bad_user" => {
                return Err(ApiError::NotFound(format!(
                    "Player {player_discord_id} not found on pemonlist"
                )));
            }
            PemonlistResponse::Err(err) => {
                return Err(ApiError::BadGateway(format!(
                    "{}: {})",
                    err.code, err.error
                )));
            }
            PemonlistResponse::Ok(player) => player,
        };

        let records = pemonlist_data
            .records
            .into_iter()
            .map(|record| {
                Ok(RecordToSync {
                    video_url: format!("https://www.youtube.com/watch?v={}", record.video_id),
                    level_id: record.level.level_id,
                    completion_time: Some(Self::parse_formatted_ms(&record.formatted_time)?),
                    mobile: record.mobile,
                })
            })
            .collect::<Result<Vec<_>, ApiError>>()?;

        Ok(records)
    }

    fn parse_formatted_ms(s: &str) -> Result<i64, ApiError> {
        let (hms, millis) = s.split_once('.').unwrap_or((s, "0"));

        let mut parts = hms.split(':');
        let hours = parts
            .next()
            .ok_or_else(|| ApiError::BadGateway("Malformed hour timestamp"))?;
        let minutes = parts
            .next()
            .ok_or_else(|| ApiError::BadGateway("Malformed minute timestamp"))?;
        let seconds = parts
            .next()
            .ok_or_else(|| ApiError::BadGateway("Malformed second timestamp"))?;
        if parts.next().is_some() {
            return Err(ApiError::BadGateway("Malformed formatted_time"));
        }

        let hours: i64 = hours
            .parse()
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse hours: {e}")))?;
        let minutes: i64 = minutes
            .parse()
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse minutes: {e}")))?;
        let seconds: i64 = seconds
            .parse()
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse seconds: {e}")))?;

        if hours < 0 || !(0..60).contains(&minutes) || !(0..60).contains(&seconds) {
            return Err(ApiError::BadGateway(
                "Malformed formatted_time (out of range)",
            ));
        }

        // normalize fraction to milliseconds
        let milliseconds = {
            let mut millis = millis.to_owned();
            if millis.len() > 3 {
                millis.truncate(3);
            } else {
                while millis.len() < 3 {
                    millis.push('0');
                }
            }
            millis
                .parse::<i64>()
                .map_err(|e| ApiError::BadGateway(format!("Failed to parse milliseconds: {e}")))?
        };

        Ok(hours * 3_600_000 + minutes * 60_000 + seconds * 1_000 + milliseconds)
    }
}
