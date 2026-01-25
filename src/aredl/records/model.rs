use std::sync::Arc;

use crate::app_data::db::{DbAppState, DbConnection};
use crate::aredl::levels::ExtendedBaseLevel;
use crate::aredl::submissions::patch::SubmissionPatchMod;
use crate::aredl::submissions::post::SubmissionPostMod;
use crate::aredl::submissions::{Submission, SubmissionStatus};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::providers::VideoProvidersAppState;
use crate::schema::{aredl::levels, aredl::records, aredl::submissions, users};
use crate::users::{user_filter, ExtendedBaseUser};
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::query_dsl::JoinOnDsl;
use diesel::{
    Connection, ExpressionMethods, Insertable, OptionalExtension, QueryDsl, RunQueryDsl,
    Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

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
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecordsQueryOptions {
    pub mobile_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<String>,
    pub sort: Option<RecordSortField>,
}
#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedRecordPage {
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
            reviewer_notes: Some("Added by a moderator".to_string()),
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
            reviewer_notes: Some("Added by a moderator".to_string()),
            ..Default::default()
        }
    }

    pub fn from_record_update(record: RecordPatch) -> Self {
        Self {
            mobile: record.mobile,
            video_url: record.video_url,
            reviewer_notes: Some("Updated by a moderator".to_string()),
            status: Some(SubmissionStatus::Accepted),
            ..Default::default()
        }
    }
}

impl RecordUpdate {
    pub fn from_record_insert(record: RecordInsert) -> Self {
        Self {
            hide_video: record.hide_video,
            is_verification: record.is_verification,
            achieved_at: record.achieved_at,
        }
    }

    pub fn from_record_patch(record: RecordPatch) -> Self {
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
        record: RecordInsert,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        let existing_submission_id = submissions::table
            .filter(submissions::submitted_by.eq(record.submitted_by))
            .filter(submissions::level_id.eq(record.level_id))
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        match existing_submission_id {
            Some(submission_id) => {
                let submission_update = (
                    SubmissionPatchMod::from_record_insert(record),
                    submissions::reviewer_id.eq(Some(authenticated.user_id)),
                );
                return Ok(diesel::update(
                    submissions::table.filter(submissions::id.eq(submission_id)),
                )
                .set((submission_update,))
                .returning(Submission::as_select())
                .get_result::<Self>(conn)?);
            }
            None => {
                let submission_insert = (
                    SubmissionPostMod::from_record_insert(record),
                    submissions::reviewer_id.eq(Some(authenticated.user_id)),
                );
                return Ok(diesel::insert_into(submissions::table)
                    .values(submission_insert)
                    .returning(Submission::as_select())
                    .get_result::<Self>(conn)?);
            }
        }
    }
}

impl Record {
    pub fn create(
        conn: &mut DbConnection,
        record: RecordInsert,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            if authenticated.user_id == record.submitted_by {
                return Err(ApiError::new(400, "You cannot create records for yourself"));
            }
            // Create the corresponding submission first and let triggers initialize the record
            let submission =
                Submission::upsert_from_record_insert(conn, record.clone(), authenticated)?;

            // Then update the record-specific fields
            let record_patch = RecordUpdate::from_record_insert(record.clone());

            let result = diesel::update(records::table)
                .filter(records::submission_id.eq(submission.id))
                .set(&record_patch)
                .returning(Record::as_select())
                .get_result::<Self>(conn)?;

            Ok(result)
        })
    }

    pub fn update(
        conn: &mut DbConnection,
        record_id: Uuid,
        record: RecordPatch,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            // Update the corresponding submission first and let triggers update the record
            let submission_patch = (
                SubmissionPatchMod::from_record_update(record.clone()),
                submissions::reviewer_id.eq(Some(authenticated.user_id)),
            );

            let (submission_id, submitted_by): (Uuid, Uuid) = records::table
                .filter(records::id.eq(record_id))
                .select((records::submission_id, records::submitted_by))
                .first(conn)?;

            if authenticated.user_id == submitted_by {
                return Err(ApiError::new(400, "You cannot update records for yourself"));
            }

            diesel::update(submissions::table)
                .filter(submissions::id.eq(submission_id))
                .set(submission_patch)
                .execute(conn)?;

            // Then update the record-specific fields
            let record_update = RecordUpdate::from_record_patch(record.clone());

            let result = match record_update == RecordUpdate::default() {
                true => records::table
                    .filter(records::id.eq(record_id))
                    .select(Record::as_select())
                    .first::<Self>(conn)?,
                false => diesel::update(records::table.filter(records::id.eq(record_id)))
                    .set(&record_update)
                    .returning(Record::as_select())
                    .get_result::<Self>(conn)?,
            };

            Ok(result)
        })
    }

    pub fn delete(
        conn: &mut DbConnection,
        record_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|conn| -> Result<(), ApiError> {
            let record = diesel::delete(records::table.filter(records::id.eq(record_id)))
                .returning(Record::as_select())
                .get_result::<Record>(conn)?;

            diesel::update(submissions::table)
                .filter(submissions::id.eq(record.submission_id))
                .filter(submissions::status.ne(SubmissionStatus::Denied)) // only update if not already denied
                .set((
                    submissions::status.eq(SubmissionStatus::Denied),
                    submissions::reviewer_id.eq(Some(authenticated.user_id)),
                    submissions::reviewer_notes
                        .eq(Some("Record removed by a moderator".to_string())),
                ))
                .execute(conn)?;
            Ok(())
        })
    }
}

// Helpers for updating the achieved_at timestamp
impl Record {
    pub async fn fetch_completion_timestamp(
        record: Record,
        providers: &VideoProvidersAppState,
    ) -> DateTime<Utc> {
        let result = async {
            let matched = providers.parse_url(&record.video_url)?;
            let metadata = providers
                .fetch_metadata(&matched)
                .await?
                .ok_or_else(|| ApiError::new(422, "Failed to fetch metadata"))?;
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
        record_id: Option<Uuid>,
        submission_id: Option<Uuid>,
        providers: &VideoProvidersAppState,
    ) -> Result<Self, ApiError> {
        let db_clone = db.clone();
        let record = web::block(move || {
            let conn = &mut db.connection()?;
            let record = match (record_id, submission_id) {
                (Some(record_id), _) => records::table
                    .filter(records::id.eq(record_id))
                    .select(Record::as_select())
                    .first::<Record>(conn)?,
                (None, Some(submission_id)) => records::table
                    .filter(records::submission_id.eq(submission_id))
                    .select(Record::as_select())
                    .first::<Record>(conn)?,
                _ => return Err(ApiError::new(400, "No record or submission ID provided")),
            };
            Ok(record)
        })
        .await??;

        if record.achieved_at < record.created_at - chrono::Duration::seconds(1) {
            return Ok(record);
        }

        let achieved_at = Record::fetch_completion_timestamp(record.clone(), providers).await;

        let result = web::block(move || -> Result<Record, ApiError> {
            let conn = &mut db_clone.connection()?;
            let result = diesel::update(records::table.filter(records::id.eq(record.id)))
                .set(records::achieved_at.eq(achieved_at))
                .returning(Record::as_select())
                .get_result::<Record>(conn)?;
            Ok(result)
        })
        .await??;

        Ok(result)
    }

    pub async fn fire_and_forget_fetch_completion_timestamp(
        db: web::Data<Arc<DbAppState>>,
        record_id: Option<Uuid>,
        submission_id: Option<Uuid>,
        providers: web::Data<Arc<VideoProvidersAppState>>,
    ) {
        tokio::spawn(async move {
            if let Err(error) =
                Record::update_timestamp(db, record_id, submission_id, providers.get_ref()).await
            {
                tracing::warn!(
                    error = %error.error_message,
                    ?record_id,
                    ?submission_id,
                    "Failed to fetch completion timestamp in background task"
                );
            }
        });
    }
}

impl ResolvedRecord {
    pub fn find(conn: &mut DbConnection, record_id: Uuid) -> Result<Self, ApiError> {
        let (record, user, level): (Record, ExtendedBaseUser, ExtendedBaseLevel) = records::table
            .filter(records::id.eq(record_id))
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
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
        options: RecordsQueryOptions,
    ) -> Result<Paginated<ResolvedRecordPage>, ApiError> {
        let build_filtered = || {
            let mut q = records::table.into_boxed::<Pg>();
            if let Some(mobile) = options.mobile_filter {
                q = q.filter(records::mobile.eq(mobile));
            }
            if let Some(level) = options.level_filter {
                q = q.filter(records::level_id.eq(level));
            }
            if let Some(ref submitter) = options.submitter_filter {
                q = q.filter(records::submitted_by.eq_any(user_filter(submitter).select(users::id)))
            }

            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let mut records_query = build_filtered()
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                Record::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ));

        if let Some(sort) = &options.sort {
            records_query = match sort {
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
            level: level,
            mobile: record.mobile,
            video_url: record.video_url,
            is_verification: record.is_verification,
            created_at: record.created_at,
            updated_at: record.updated_at,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
        }
    }
}
