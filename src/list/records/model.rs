use std::sync::Arc;

use crate::app_data::db::{DbAppState, DbConnection};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::list::levels::{ExtendedBaseLevel, Level};
use crate::list::submissions::patch::SubmissionPatchMod;
use crate::list::submissions::post::SubmissionPostMod;
use crate::list::submissions::{Submission, SubmissionStatus};
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use crate::providers::ProvidersAppState;
use crate::schema::{levels, records, submissions, users};
use crate::users::badges::UserBadge;
use crate::users::{user_filter, ExtendedBaseUser};
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::dsl::count;
use diesel::pg::Pg;
use diesel::{Insertable, Selectable};
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema, Clone)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct Record {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Internal UUID of the submission this record is linked to.
    pub submission_id: Uuid,
    /// Level this record is for.
    pub level_id: Uuid,
    /// User who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds. Only present for platformer records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<i64>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Internal UUID of the submission this record is linked to.
    pub submission_id: Uuid,
    /// Level this record is for.
    pub level: ExtendedBaseLevel,
    /// User who submitted the record.
    pub submitted_by: ExtendedBaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds. Only present for platformer records.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_time: Option<i64>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema, Clone)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct RecordInsert {
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Internal UUID of the level the record is for.
    pub level_id: Uuid,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds. Only present for platformer records.
    pub completion_time: Option<i64>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: Option<bool>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema, Clone)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct RecordPatch {
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Option<Uuid>,
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: Option<bool>,
    /// Completion time of the record in milliseconds. Only present for platformer records.
    #[serde(default, with = "double_option")]
    pub completion_time: Option<Option<i64>>,
    /// Internal UUID of the level the record is for.
    pub level_id: Option<Uuid>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema, Default, PartialEq)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct RecordUpdate {
    /// Whether the record's video should be hidden on the website.
    pub hide_video: Option<bool>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub enum RecordSortField {
    OldestCreatedAt,
    NewestCreatedAt,
    OldestAchievedAt,
    NewestAchievedAt,
    OldestUpdatedAt,
    NewestUpdatedAt,
    /// Only available for platformer.
    ShortestCompletionTime,
    /// Only available for platformer.
    LongestCompletionTime,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecordsQueryOptions {
    pub mobile_filter: Option<bool>,
    pub verification_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<String>,
    pub sort: Option<RecordSortField>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MutualVictorsQuery {
    pub level_id: String,
    pub other_level_id: String,
    pub high_extremes: Option<bool>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct MutualVictors {
    /// The first level to find victors from
    pub level: Level,
    /// The second level to find mutual victors of the first level from
    pub other_level: Level,
    /// The resulting list of users who have a record on both levels
    pub mutuals: Vec<ExtendedBaseUser>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedRecordPage {
    /// Resolved records for this page
    data: Vec<ResolvedRecord>,
}

impl SubmissionPostMod {
    pub fn from_record_insert(record: RecordInsert) -> Self {
        Self {
            submitted_by: Some(record.submitted_by),
            level_id: record.level_id,
            mobile: record.mobile,
            video_url: record.video_url,
            status: Some(SubmissionStatus::Accepted),
            reviewer_notes: Some("Added by a moderator".to_owned()),
            completion_time: record.completion_time,
            ..Default::default()
        }
    }
}

impl SubmissionPatchMod {
    pub fn from_record_insert(record: RecordInsert) -> Self {
        Self {
            mobile: Some(record.mobile),
            video_url: Some(record.video_url),
            status: Some(SubmissionStatus::Accepted),
            reviewer_notes: Some(Some("Added by a moderator".to_owned())),
            completion_time: Some(record.completion_time),
            ..Default::default()
        }
    }

    pub fn from_record_update(record: RecordPatch) -> Self {
        Self {
            mobile: record.mobile,
            video_url: record.video_url,
            reviewer_notes: Some(Some("Updated by a moderator".to_owned())),
            status: Some(SubmissionStatus::Accepted),
            completion_time: record.completion_time,
            ..Default::default()
        }
    }
}

impl RecordUpdate {
    pub fn from_record_insert(record: &RecordInsert) -> Self {
        Self {
            hide_video: record.hide_video,
            is_verification: record.is_verification,
            achieved_at: record.achieved_at,
        }
    }

    pub fn from_record_patch(record: &RecordPatch) -> Self {
        Self {
            hide_video: record.hide_video,
            is_verification: record.is_verification,
            achieved_at: record.achieved_at,
        }
    }
}

impl Submission {
    pub fn upsert_from_record_insert(
        conn: &mut DbConnection,
        list: List,
        record: RecordInsert,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        let existing_submission_id = submissions::table
            .filter(submissions::list_id.eq(list))
            .filter(submissions::submitted_by.eq(record.submitted_by))
            .filter(submissions::level_id.eq(record.level_id))
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        if let Some(submission_id) = existing_submission_id {
            let submission_update = (
                SubmissionPatchMod::from_record_insert(record),
                submissions::reviewer_id.eq(Some(authenticated.user_id)),
            );
            Ok(
                diesel::update(submissions::table.filter(submissions::id.eq(submission_id)))
                    .set(submission_update)
                    .returning(Submission::as_select())
                    .get_result::<Self>(conn)?,
            )
        } else {
            let submission_insert = (
                SubmissionPostMod::from_record_insert(record),
                submissions::reviewer_id.eq(Some(authenticated.user_id)),
            );
            Ok(diesel::insert_into(submissions::table)
                .values((submissions::list_id.eq(list), submission_insert))
                .returning(Submission::as_select())
                .get_result::<Self>(conn)?)
        }
    }
}

impl Record {
    pub fn create(
        conn: &mut DbConnection,
        list: List,
        record: &RecordInsert,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            if authenticated.user_id == record.submitted_by {
                return Err(ApiError::Forbidden(
                    "You cannot create records for yourself",
                ));
            }
            // Create the corresponding submission first and let triggers initialize the record
            let submission =
                Submission::upsert_from_record_insert(conn, list, record.clone(), authenticated)?;

            // Then update the record-specific fields
            let record_patch = RecordUpdate::from_record_insert(record);

            let result = diesel::update(records::table)
                .filter(records::list_id.eq(list))
                .filter(records::submission_id.eq(submission.id))
                .set(&record_patch)
                .returning(Record::as_select())
                .get_result::<Self>(conn)?;

            UserBadge::update_user_badges(conn, result.submitted_by)?;

            Ok(result)
        })
    }

    pub fn update(
        conn: &mut DbConnection,
        list: List,
        record_id: Uuid,
        record: &RecordPatch,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            // Update the corresponding submission first and let triggers update the record
            let submission_patch = (
                SubmissionPatchMod::from_record_update(record.clone()),
                submissions::reviewer_id.eq(Some(authenticated.user_id)),
            );

            let (submission_id, submitted_by): (Uuid, Uuid) = records::table
                .filter(records::list_id.eq(list))
                .filter(records::id.eq(record_id))
                .select((records::submission_id, records::submitted_by))
                .first(conn)?;

            if authenticated.user_id == submitted_by {
                return Err(ApiError::Forbidden(
                    "You cannot update records for yourself",
                ));
            }

            diesel::update(submissions::table)
                .filter(submissions::list_id.eq(list))
                .filter(submissions::id.eq(submission_id))
                .set(submission_patch)
                .execute(conn)?;

            // Then update the record-specific fields
            let record_update = RecordUpdate::from_record_patch(record);

            let result = if record_update == RecordUpdate::default() {
                records::table
                    .filter(records::list_id.eq(list))
                    .filter(records::id.eq(record_id))
                    .select(Record::as_select())
                    .first::<Self>(conn)?
            } else {
                diesel::update(
                    records::table
                        .filter(records::list_id.eq(list))
                        .filter(records::id.eq(record_id)),
                )
                .set(&record_update)
                .returning(Record::as_select())
                .get_result::<Self>(conn)?
            };

            UserBadge::update_user_badges(conn, submitted_by)?;

            Ok(result)
        })
    }

    pub fn delete(
        conn: &mut DbConnection,
        list: List,
        record_id: Uuid,
        authenticated: &Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|conn| -> Result<(), ApiError> {
            let record = diesel::delete(
                records::table
                    .filter(records::list_id.eq(list))
                    .filter(records::id.eq(record_id)),
            )
            .returning(Record::as_select())
            .get_result::<Record>(conn)?;

            diesel::update(submissions::table)
                .filter(submissions::list_id.eq(list))
                .filter(submissions::id.eq(record.submission_id))
                .filter(submissions::status.ne(SubmissionStatus::Denied)) // only update if not already denied
                .set((
                    submissions::status.eq(SubmissionStatus::Denied),
                    submissions::reviewer_id.eq(Some(authenticated.user_id)),
                    submissions::reviewer_notes
                        .eq(Some("Record removed by a moderator".to_owned())),
                ))
                .execute(conn)?;
            Ok(())
        })
    }
}

impl MutualVictors {
    pub fn find(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        other_level_id: Uuid,
        high_extremes: Option<bool>,
    ) -> Result<Self, ApiError> {
        let level = levels::table
            .filter(levels::list_id.eq(list))
            .filter(levels::id.eq(level_id))
            .select(Level::as_select())
            .first::<Level>(conn)?;
        let other_level = levels::table
            .filter(levels::list_id.eq(list))
            .filter(levels::id.eq(other_level_id))
            .select(Level::as_select())
            .first::<Level>(conn)?;

        let other_level_victors = records::table
            .filter(records::list_id.eq(list))
            .filter(records::level_id.eq(other_level_id))
            .select(records::submitted_by)
            .load::<Uuid>(conn)?;

        let mut mutuals_query = records::table
            .filter(records::list_id.eq(list))
            .filter(records::level_id.eq(level_id))
            .filter(records::submitted_by.eq_any(other_level_victors))
            .inner_join(users::table)
            .into_boxed();

        if let Some(true) = high_extremes {
            let users_high_extremes = records::table
                .filter(records::list_id.eq(list))
                .group_by(records::submitted_by)
                .having(count(records::id).gt(50))
                .select(records::submitted_by)
                .load::<Uuid>(conn)?;

            mutuals_query = mutuals_query.filter(records::submitted_by.eq_any(users_high_extremes));
        }

        let mutuals = mutuals_query
            .select(ExtendedBaseUser::as_select())
            .order_by(records::achieved_at.asc())
            .load::<ExtendedBaseUser>(conn)?;

        Ok(Self {
            level,
            other_level,
            mutuals,
        })
    }
}

// Helpers for updating the achieved_at timestamp
impl Record {
    pub async fn fetch_completion_timestamp(
        record: Record,
        providers: &ProvidersAppState,
    ) -> DateTime<Utc> {
        let result = async {
            let matched = providers.parse_url(&record.video_url)?;
            let metadata = providers
                .fetch_metadata(&matched)
                .await?
                .ok_or_else(|| ApiError::BadGateway("Failed to fetch metadata"))?;
            Ok::<_, ApiError>(metadata.published_at)
        }
        .await;

        match result {
            Ok(Some(timestamp)) => timestamp,
            Ok(None) => {
                tracing::warn!(
                    %record.id,
                    %record.video_url,
                    "Fetched metadata does not contain publication timestamp"
                );
                record.created_at
            }
            Err(e) => {
                tracing::warn!(
                    error = %e.error_message,
                    %record.id,
                    %record.video_url,
                    "Failed to fetch metadata"
                );
                record.created_at
            }
        }
    }

    pub async fn update_timestamp(
        db: web::Data<Arc<DbAppState>>,
        list: List,
        record_id: Uuid,
        providers: &ProvidersAppState,
    ) -> Result<Self, ApiError> {
        let db_clone = db.clone();
        let record: Record = web::block(move || {
            let conn = &mut db.connection()?;
            let record = records::table
                .filter(records::list_id.eq(list))
                .filter(records::id.eq(record_id))
                .select(Record::as_select())
                .first::<Record>(conn)?;

            Ok::<Record, ApiError>(record)
        })
        .await??;

        if record.achieved_at < record.created_at - chrono::Duration::seconds(1) {
            return Ok(record);
        }

        let achieved_at = Record::fetch_completion_timestamp(record.clone(), providers).await;

        let result = web::block(move || -> Result<Record, ApiError> {
            let conn = &mut db_clone.connection()?;
            let result = diesel::update(
                records::table
                    .filter(records::list_id.eq(list))
                    .filter(records::id.eq(record.id)),
            )
            .set(records::achieved_at.eq(achieved_at))
            .returning(Record::as_select())
            .get_result::<Record>(conn)?;
            Ok(result)
        })
        .await??;

        Ok(result)
    }

    pub fn post_accept_actions(
        db: web::Data<Arc<DbAppState>>,
        list: List,
        submissions: &[Submission],
        providers: web::Data<Arc<ProvidersAppState>>,
    ) {
        let accepted: Vec<_> = submissions
            .iter()
            .filter(|submission| submission.status == SubmissionStatus::Accepted)
            .map(|submission| (submission.id, submission.submitted_by))
            .collect();
        if accepted.is_empty() {
            return;
        }

        tokio::spawn(async move {
            let mut users = std::collections::HashSet::new();
            let mut processed = std::collections::HashSet::new();
            for (submission_id, submitted_by) in accepted {
                users.insert(submitted_by);
                if !processed.insert(submission_id) {
                    continue;
                }
                let db_clone = db.clone();
                let record = match web::block(move || {
                    Record::find_from_submission(&mut db_clone.connection()?, list, submission_id)
                })
                .await
                {
                    Ok(Ok(record)) => record,
                    result => {
                        tracing::warn!(
                            ?result,
                            ?submission_id,
                            "Failed to load record for post submission accept actions"
                        );
                        continue;
                    }
                };

                let record = match Record::update_timestamp(
                    db.clone(),
                    list,
                    record.id,
                    providers.get_ref(),
                )
                .await
                {
                    Ok(updated_record) => updated_record,
                    Err(error) => {
                        tracing::warn!(error = %error.error_message, ?record.id, ?submission_id,
                            "Failed to update record's achieved_at timestamp for post submission accept actions");
                        record
                    }
                };

                let db_clone = db.clone();
                let record_id = record.id;
                if let Err(error) = async {
                    web::block(move || {
                        record.complete_bounty_if_exists(&mut db_clone.connection()?, list)
                    })
                    .await??;
                    Ok::<(), ApiError>(())
                }
                .await
                {
                    tracing::warn!(error = %error.error_message, ?record_id,
                        "Failed to complete bounty for post submission accept actions");
                }
            }

            // refresh users badges once
            if let Err(error) = web::block(move || {
                for user_id in users {
                    if let Err(error) = db
                        .connection()
                        .and_then(|mut conn| UserBadge::update_user_badges(&mut conn, user_id))
                    {
                        tracing::warn!(error = %error.error_message, ?user_id,
                            "Failed to update user badges for post submission accept actions");
                    }
                }
            })
            .await
            {
                tracing::warn!(%error, "Failed to run badge updates for post submission accept actions");
            }
        });
    }
}

impl Record {
    pub fn find_from_submission(
        conn: &mut DbConnection,
        list: List,
        submission_id: Uuid,
    ) -> Result<Self, ApiError> {
        let record = records::table
            .filter(records::list_id.eq(list))
            .filter(records::submission_id.eq(submission_id))
            .select(Record::as_select())
            .first::<Record>(conn)?;

        Ok(record)
    }
}

impl ResolvedRecord {
    pub fn find(conn: &mut DbConnection, list: List, record_id: Uuid) -> Result<Self, ApiError> {
        let (record, user, level): (Record, ExtendedBaseUser, ExtendedBaseLevel) = records::table
            .filter(records::list_id.eq(list))
            .filter(records::id.eq(record_id))
            .inner_join(users::table)
            .inner_join(levels::table)
            .order_by(records::completion_time.asc())
            .select((
                Record::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .first(conn)?;

        Ok(Self::from_data(record, user, level))
    }

    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        list: List,
        options: &RecordsQueryOptions,
    ) -> Result<Paginated<ResolvedRecordPage>, ApiError> {
        let build_filtered = || {
            let mut q = records::table
                .filter(records::list_id.eq(list))
                .into_boxed::<Pg>();
            if let Some(mobile) = options.mobile_filter {
                q = q.filter(records::mobile.eq(mobile));
            }
            if let Some(verification) = options.verification_filter {
                q = q.filter(records::is_verification.eq(verification));
            }
            if let Some(level) = options.level_filter {
                q = q.filter(records::level_id.eq(level));
            }
            if let Some(submitter) = &options.submitter_filter {
                q = q
                    .filter(records::submitted_by.eq_any(user_filter(submitter).select(users::id)));
            }
            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let mut records_query = build_filtered()
            .inner_join(users::table)
            .inner_join(levels::table)
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                Record::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ));

        if let Some(sort) = &options.sort {
            records_query = match sort {
                RecordSortField::ShortestCompletionTime
                | RecordSortField::LongestCompletionTime
                    if list != List::Platformer =>
                {
                    return Err(ApiError::BadRequest(
                        "Completion-time sorting is only available for platformer",
                    ));
                }

                RecordSortField::OldestCreatedAt => {
                    records_query.order_by(records::created_at.asc())
                }
                RecordSortField::NewestCreatedAt => {
                    records_query.order_by(records::created_at.desc())
                }
                RecordSortField::OldestAchievedAt => {
                    records_query.order_by(records::achieved_at.asc())
                }
                RecordSortField::NewestAchievedAt => {
                    records_query.order_by(records::achieved_at.desc())
                }
                RecordSortField::OldestUpdatedAt => {
                    records_query.order_by(records::updated_at.asc())
                }
                RecordSortField::NewestUpdatedAt => {
                    records_query.order_by(records::updated_at.desc())
                }
                RecordSortField::ShortestCompletionTime => {
                    records_query.order_by(records::completion_time.asc())
                }
                RecordSortField::LongestCompletionTime => {
                    records_query.order_by(records::completion_time.desc())
                }
            };
        } else {
            records_query = records_query.order_by(records::created_at.desc());
        }

        let records = records_query.load::<(Record, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?;
        let records_resolved: Vec<Self> = records
            .into_iter()
            .map(|(record, user, level)| Self::from_data(record, user, level))
            .collect();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            ResolvedRecordPage {
                data: records_resolved,
            },
        ))
    }

    pub fn from_data(record: Record, user: ExtendedBaseUser, level: ExtendedBaseLevel) -> Self {
        Self {
            id: record.id,
            submission_id: record.submission_id,
            submitted_by: user,
            level,
            mobile: record.mobile,
            video_url: record.video_url,
            completion_time: record.completion_time,
            is_verification: record.is_verification,
            created_at: record.created_at,
            updated_at: record.updated_at,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
        }
    }
}
