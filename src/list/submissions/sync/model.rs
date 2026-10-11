use chrono::Utc;
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    list::{
        levels::LevelStatus,
        submissions::{sync::pemonlist::Pemonlist, Submission, SubmissionStatus},
        List,
    },
    schema::{levels, submissions},
    users::User,
};

use diesel::prelude::*;
use std::collections::HashMap;

#[derive(Display, Clone, Copy, ToSchema, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RecordsSyncProvider {
    Pemonlist,
}
pub struct RecordsSync;

pub struct RecordsSyncResult {
    pub submissions: Vec<Submission>,
    pub changed_submissions: Vec<Submission>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema, Clone)]
pub struct RecordToSync {
    pub video_url: String,
    pub level_id: i32,
    pub completion_time: Option<i64>,
    pub mobile: bool,
}

impl RecordsSync {
    pub fn sync(
        provider: RecordsSyncProvider,
        list: List,
        user: &User,
    ) -> Result<Vec<RecordToSync>, ApiError> {
        let imported_records: Vec<RecordToSync> = match (provider, list) {
            (RecordsSyncProvider::Pemonlist, List::Platformer) => Pemonlist::sync(user.clone())?,
            (_, _) => {
                return Err(ApiError::BadRequest(
                    "This provider is not supported to sync records for this list",
                ));
            }
        };

        Ok(imported_records)
    }

    pub fn import_records(
        conn: &mut DbConnection,
        provider: RecordsSyncProvider,
        list: List,
        user: &User,
        records_to_import: Vec<RecordToSync>,
    ) -> Result<RecordsSyncResult, ApiError> {
        // Keep the last record supplied for each level.
        let records_to_import = records_to_import
            .into_iter()
            .map(|record| (record.level_id, record))
            .collect::<HashMap<_, _>>();

        // Fetch and parse provider data before opening the database transaction.
        conn.transaction(|conn| {
            let placed_levels = levels::table
                .filter(levels::list_id.eq(list))
                .filter(levels::status.ne(LevelStatus::Removed))
                .filter(levels::level_id.eq_any(records_to_import.keys().copied()))
                .select((levels::level_id, levels::id))
                .load::<(i32, Uuid)>(conn)?
                .into_iter()
                .collect::<HashMap<i32, Uuid>>();

            let existing_submissions = submissions::table
                .inner_join(levels::table)
                .filter(submissions::submitted_by.eq(user.id))
                .filter(submissions::level_id.eq_any(placed_levels.values().copied()))
                .select((levels::level_id, Submission::as_select()))
                .load::<(i32, Submission)>(conn)?
                .into_iter()
                .collect::<HashMap<i32, Submission>>();

            let now = Utc::now();

            let mut imported = Vec::new();
            let mut changed_submissions = Vec::new();

            for imported_record in records_to_import.into_values() {
                let existing_submission = existing_submissions.get(&imported_record.level_id);
                let Some(placed_level_id) = placed_levels.get(&imported_record.level_id) else {
                    continue;
                };

                // ensure there is an Accepted submission with the correct data
                // if exists, only update if relevant data has changed (avoid triggering a submission history log entry)

                let submission = if let Some(existing_submission) = existing_submission {
                    if existing_submission.status != SubmissionStatus::Accepted
                        || existing_submission.completion_time != imported_record.completion_time
                        || existing_submission.mobile != imported_record.mobile
                        || existing_submission.video_url != imported_record.video_url
                    {
                        diesel::update(
                            submissions::table.filter(submissions::id.eq(existing_submission.id)),
                        )
                        .set((
                            submissions::mobile.eq(imported_record.mobile),
                            submissions::video_url.eq(imported_record.video_url),
                            submissions::completion_time.eq(imported_record.completion_time),
                            submissions::status.eq(SubmissionStatus::Accepted),
                            submissions::reviewer_id.eq::<Option<uuid::Uuid>>(None),
                            submissions::reviewer_notes.eq::<Option<String>>(Some(format!(
                                "Accepted via {provider} sync"
                            ))),
                            submissions::updated_at.eq(now),
                        ))
                        .returning(Submission::as_select())
                        .get_result::<Submission>(conn)?
                    } else {
                        imported.push(existing_submission.clone());
                        continue;
                    }
                } else {
                    diesel::insert_into(submissions::table)
                        .values((
                            submissions::list_id.eq(list),
                            submissions::submitted_by.eq(user.id),
                            submissions::level_id.eq(placed_level_id),
                            submissions::mobile.eq(imported_record.mobile),
                            submissions::custom_copy_id.eq::<Option<i32>>(None),
                            submissions::video_url.eq(imported_record.video_url),
                            submissions::raw_url.eq::<Option<String>>(None),
                            submissions::mod_menu.eq::<Option<String>>(Some(String::from("None"))),
                            submissions::user_notes.eq::<Option<String>>(None),
                            submissions::priority.eq(false),
                            submissions::status.eq(SubmissionStatus::Accepted),
                            submissions::completion_time.eq(imported_record.completion_time),
                            submissions::reviewer_id.eq::<Option<uuid::Uuid>>(None),
                            submissions::reviewer_notes.eq::<Option<String>>(Some(format!(
                                "Accepted via {provider} sync"
                            ))),
                            submissions::created_at.eq(now),
                            submissions::updated_at.eq(now),
                        ))
                        .returning(Submission::as_select())
                        .get_result::<Submission>(conn)?
                };

                changed_submissions.push(submission.clone());
                imported.push(submission);
            }

            Ok(RecordsSyncResult {
                submissions: imported,
                changed_submissions,
            })
        })
    }
}
