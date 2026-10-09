use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::records::Record;
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{records, users};
use crate::users::{user_ilike_filter, BaseUser, ExtendedBaseUser};
use chrono::{DateTime, Utc};
use diesel::dsl::count;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(utoipa::ToSchema, Serialize, Deserialize, Debug)]
pub struct RecordQuery {
    high_extremes: Option<bool>,
    submitter_filter: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
/// A resolved record for a specific level (ommits the level field compared to `ResolvedRecord`).
pub struct LevelResolvedRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: BaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<i64>,
    /// Timestamp of when this record was achieved.
    pub achieved_at: DateTime<Utc>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
/// A resolved record for a specific level (ommits the level field compared to `ResolvedRecord`), with an extended resolved user.
pub struct LevelResolvedRecordExtended {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: ExtendedBaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<i64>,
    /// Timestamp of when this record was achieved.
    pub achieved_at: DateTime<Utc>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct LevelResolvedRecordPage {
    /// Resolved level records for this page
    pub data: Vec<LevelResolvedRecordExtended>,
}

impl LevelResolvedRecord {
    pub fn from_data(record: Record, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            completion_time: record.completion_time,
            achieved_at: record.achieved_at,
            hide_video: record.hide_video,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

impl LevelResolvedRecordExtended {
    pub fn find_all_by_level<const D: i64>(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        page_query: PageQuery<D>,
        opts: &RecordQuery,
    ) -> Result<Paginated<LevelResolvedRecordPage>, ApiError> {
        let build_filtered = || {
            let mut query = records::table
                .filter(records::list_id.eq(list))
                .filter(records::level_id.eq(level_id))
                .filter(records::is_verification.eq(false))
                .inner_join(users::table)
                .filter(users::ban_level.le(2))
                .into_boxed();

            if let Some(submitter_filter) = &opts.submitter_filter {
                query = query.filter(
                    users::id.eq_any(user_ilike_filter(submitter_filter).select(users::id)),
                );
            }

            if let Some(true) = opts.high_extremes {
                let counted_records = diesel::alias!(records as counted_records);
                let users_high_extremes = counted_records
                    .filter(counted_records.field(records::list_id).eq(list))
                    .group_by(counted_records.field(records::submitted_by))
                    .having(count(counted_records.field(records::id)).gt(50))
                    .select(counted_records.field(records::submitted_by));

                query = query.filter(records::submitted_by.eq_any(users_high_extremes));
            }

            query
        };

        let total_count = build_filtered().count().get_result::<i64>(conn)?;

        let query = match list {
            List::Classic => {
                build_filtered().order((records::achieved_at.asc(), records::id.asc()))
            }
            List::Platformer => {
                build_filtered().order((records::completion_time.asc(), records::id.asc()))
            }
        };

        let records = query
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((Record::as_select(), ExtendedBaseUser::as_select()))
            .load::<(Record, ExtendedBaseUser)>(conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            LevelResolvedRecordPage {
                data: records_resolved,
            },
        ))
    }

    pub fn from_data(record: Record, user: ExtendedBaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            hide_video: record.hide_video,
            completion_time: record.completion_time,
            achieved_at: record.achieved_at,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}
