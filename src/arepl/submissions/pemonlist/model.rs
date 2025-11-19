use chrono::{NaiveTime, Timelike, Utc};
use diesel::prelude::*;
use serde::Deserialize;

use crate::{
    app_data::db::DbConnection,
    arepl::{
        levels::ExtendedBaseLevel,
        submissions::{Submission, SubmissionStatus},
    },
    auth::Authenticated,
    error_handler::ApiError,
    schema::{
        arepl::{levels, submissions},
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
    pub fn sync_with_pemonlist(
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<Vec<Submission>, ApiError> {
        let player_discord_id = users::table
            .filter(users::id.eq(authenticated.user_id))
            .select(users::discord_id)
            .first::<Option<String>>(conn)?;

        if player_discord_id.is_none() {
            return Err(ApiError::new(400, "Given user does not have a discord id"));
        }

        let player_discord_id = player_discord_id.unwrap();

        let client = reqwest::blocking::Client::new();
        let base_url = std::env::var("PEMONLIST_API_URL")
            .unwrap_or_else(|_| "https://pemonlist.com/api/player".to_string());
        let url = format!("{}/{}", base_url.trim_end_matches('/'), player_discord_id);
        let resp = client
            .get(&url)
            .send()
            .map_err(|e| ApiError::new(500, &e.to_string()))?;

        let pemonlist_response: PemonlistResponse = resp.json().map_err(|e| {
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

            // find existing submission for this user/level in arepl.submissions
            let existing_submission: Option<Submission> = submissions::table
                .filter(submissions::submitted_by.eq(authenticated.user_id))
                .filter(submissions::level_id.eq(existing_level.id))
                .select(Submission::as_select())
                .first::<Submission>(conn)
                .optional()?;

            let now = Utc::now();
            let timestamp = Self::parse_formatted_ms(&pemonlist_record.formatted_time)?;

            let video_url = format!("https://youtu.be/{}", pemonlist_record.video_id);

            // ensure there is an Accepted submission with the correct data
            let submission = if let Some(old) = existing_submission {
                diesel::update(submissions::table.filter(submissions::id.eq(old.id)))
                    .set((
                        submissions::mobile.eq(pemonlist_record.mobile),
                        submissions::video_url.eq(video_url),
                        submissions::completion_time.eq(timestamp),
                        submissions::status.eq(SubmissionStatus::Accepted),
                        submissions::reviewer_id.eq::<Option<uuid::Uuid>>(None),
                        submissions::reviewer_notes.eq::<Option<String>>(None),
                        submissions::updated_at.eq(now),
                    ))
                    .returning(Submission::as_select())
                    .get_result::<Submission>(conn)?
            } else {
                diesel::insert_into(submissions::table)
                    .values((
                        submissions::submitted_by.eq(authenticated.user_id),
                        submissions::level_id.eq(existing_level.id),
                        submissions::mobile.eq(pemonlist_record.mobile),
                        submissions::ldm_id.eq::<Option<i32>>(None),
                        submissions::video_url.eq(video_url),
                        submissions::raw_url.eq::<Option<String>>(None),
                        submissions::mod_menu.eq::<Option<String>>(None),
                        submissions::user_notes.eq::<Option<String>>(None),
                        submissions::priority.eq(false),
                        submissions::status.eq(SubmissionStatus::Accepted),
                        submissions::completion_time.eq(timestamp),
                        submissions::reviewer_id.eq::<Option<uuid::Uuid>>(None),
                        submissions::reviewer_notes.eq::<Option<String>>(None),
                        submissions::created_at.eq(now),
                        submissions::updated_at.eq(now),
                    ))
                    .returning(Submission::as_select())
                    .get_result::<Submission>(conn)?
            };

            imported.push(submission);
        }

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
