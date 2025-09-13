use actix_web::web;
use chrono::{NaiveTime, Timelike, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use crate::{
    arepl::{
        levels::ExtendedBaseLevel,
        records::{Record, RecordInsert, RecordUpdate},
    },
    auth::Authenticated,
    db::DbAppState,
    error_handler::ApiError,
    schema::{
        arepl::{levels, records},
        users,
    },
};

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

impl PemonlistPlayer {
    pub async fn sync_with_pemonlist(
        db: web::Data<Arc<DbAppState>>,
        authenticated: Authenticated,
    ) -> Result<Vec<Record>, ApiError> {
        let conn = &mut db.connection()?;

        let player_discord_id = users::table
            .filter(users::id.eq(authenticated.user_id))
            .select(users::discord_id)
            .first::<Option<String>>(conn)?;

        if player_discord_id.is_none() {
            return Err(ApiError::new(400, "Given user does not have a discord id"));
        }

        let player_discord_id = player_discord_id.unwrap();

        let client = reqwest::Client::new();
        let base_url = std::env::var("PEMONLIST_API_URL")
            .unwrap_or_else(|_| "https://pemonlist.com/api/player".to_string());
        let url = format!("{}/{}", base_url.trim_end_matches('/'), player_discord_id);
        let resp = client
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::new(500, &e.to_string()))?;

        let pemonlist_response: PemonlistResponse = resp.json().await.map_err(|e| {
            ApiError::new(
                500,
                &format!(
                    "Failed to parse data received from pemonlist: {}",
                    e.to_string()
                ),
            )
        })?;

        let pemonlist_data = match pemonlist_response {
            PemonlistResponse::Err(err) if err.error && err.code == "bad_user" => {
                return Err(ApiError::new(
                    404,
                    &format!("Player {} not found on pemonlist", player_discord_id),
                ));
            }
            PemonlistResponse::Err(err) => {
                return Err(ApiError::new(500, &format!("{}: {})", err.code, err.error)));
            }
            PemonlistResponse::Ok(player) => player,
        };

        let imported = web::block(move || -> Result<Vec<Record>, ApiError> {
            let conn = &mut db.connection()?;

            let mut imported = Vec::new();

            for pemonlist_record in pemonlist_data.records {
                let existing_level = levels::table
                    .filter(levels::level_id.eq(pemonlist_record.level.level_id))
                    .select(ExtendedBaseLevel::as_select())
                    .first::<ExtendedBaseLevel>(conn)
                    .optional()?;

                if existing_level.is_none() {
                    continue;
                }

                let existing_level = existing_level.unwrap();

                let existing_record: Option<Record> = records::table
                    .filter(records::submitted_by.eq(authenticated.user_id))
                    .filter(records::level_id.eq(existing_level.id))
                    .select(Record::as_select())
                    .first::<Record>(conn)
                    .optional()?;

                let now = Utc::now();

                let timestamp = Self::parse_formatted_ms(&pemonlist_record.formatted_time)?;

                let saved = if let Some(old) = existing_record {
                    let update = RecordUpdate {
                        submitted_by: None,
                        mobile: Some(pemonlist_record.mobile),
                        ldm_id: None,
                        video_url: Some(format!(
                            "https://youtu.be/{}",
                            pemonlist_record.video_id.clone()
                        )),
                        hide_video: false,
                        level_id: None,
                        completion_time: Some(timestamp),
                        is_verification: Some(false),
                        raw_url: None,
                        updated_at: Some(now),
                        created_at: None,
                    };
                    Record::update(db.clone(), old.id, update)?
                } else {
                    let ins = RecordInsert {
                        submitted_by: authenticated.user_id,
                        mobile: pemonlist_record.mobile,
                        ldm_id: None,
                        level_id: existing_level.id,
                        video_url: format!(
                            "https://youtu.be/{}",
                            pemonlist_record.video_id.clone()
                        ),
                        hide_video: false,
                        completion_time: timestamp,
                        is_verification: Some(false),
                        raw_url: None,
                        reviewer_id: None,
                        created_at: Some(now),
                        updated_at: Some(now),
                    };
                    Record::create(db.clone(), ins)?
                };

                imported.push(saved);
            }
            Ok(imported)
        })
        .await??;

        Ok(imported)
    }

    fn parse_formatted_ms(s: &str) -> Result<i64, ApiError> {
        let mut parts = s.split('.');
        let time_part = parts
            .next()
            .ok_or_else(|| ApiError::new(500, "Malformed formatted_time"))?;
        let ms_part = parts.next().unwrap_or("0");
        let t = NaiveTime::parse_from_str(time_part, "%H:%M:%S")
            .map_err(|e| ApiError::new(500, &format!("Time parse error: {}", e)))?;
        let ms = {
            let mut ms = ms_part.to_string();
            if ms.len() > 3 {
                ms.truncate(3);
            } else {
                while ms.len() < 3 {
                    ms.push('0');
                }
            }
            ms.parse::<i64>()
                .map_err(|e| ApiError::new(500, &format!("MS parse error: {}", e)))?
        };
        Ok((t.hour() as i64) * 3_600_000
            + (t.minute() as i64) * 60_000
            + (t.second() as i64) * 1_000
            + ms)
    }
}
