use crate::app_data::db::DbConnection;
use crate::arepl::levels::ExtendedBaseLevel;
use crate::arepl::submissions::patch::SubmissionPatchMod;
use crate::arepl::submissions::post::SubmissionInsert;
use crate::arepl::submissions::{Submission, SubmissionStatus};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{arepl::levels, arepl::records, arepl::submissions, users};
use crate::users::{BaseUser, ExtendedBaseUser};
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

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct Record {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Level ID in the game. May not be unique for 2P levels.
    pub level_id: Uuid,
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema, Clone, Default)]
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
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
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
    /// Completion time of the record in milliseconds.
    pub completion_time: Option<i64>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
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
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct PublicRecordTemplate<T> {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: T,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
}

pub type PublicRecordUnresolved = PublicRecordTemplate<Uuid>;
pub type PublicRecordResolved = PublicRecordTemplate<BaseUser>;
pub type PublicRecordResolvedExtended = PublicRecordTemplate<ExtendedBaseUser>;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=records, check_for_backend(Pg))]
#[schema(bound = "LevelT: utoipa::ToSchema, UserT: utoipa::ToSchema")]
pub struct FullRecordTemplate<LevelT, UserT> {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Level this record is for.
    #[serde(rename = "level")]
    pub level_id: LevelT,
    /// User who submitted the record.
    pub submitted_by: UserT,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

pub type FullRecordUnresolved = FullRecordTemplate<Uuid, Uuid>;
pub type FullRecordResolved = FullRecordTemplate<ExtendedBaseLevel, BaseUser>;

// Weird shenanigans type to get the FullRecordTemplate with UUID to work with ToSchema for Utoipa.
#[derive(Serialize, Deserialize, ToSchema)]
#[schema(title = "FullRecordUnresolved")]
pub struct FullRecordUnresolvedDto {
    #[serde(flatten)]
    #[schema(inline)]
    pub record: FullRecordTemplate<Uuid, Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecordsQueryOptions {
    pub mobile_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FullUnresolvedRecordPage {
    data: Vec<FullRecordUnresolvedDto>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FullResolvedRecordPage {
    data: Vec<FullRecordResolved>,
}

impl SubmissionInsert {
    pub fn from_record_insert(record: RecordInsert) -> Self {
        Self {
            submitted_by: record.submitted_by,
            level_id: record.level_id,
            mobile: record.mobile,
            video_url: record.video_url,
            status: SubmissionStatus::Accepted,
            ..Default::default()
        }
    }
}

impl SubmissionPatchMod {
    pub fn from_record_insert(record: RecordInsert) -> Self {
        Self {
            mobile: Some(record.mobile),
            video_url: Some(record.video_url),
            reviewer_notes: Some("Added by a moderator".to_string()),
            status: Some(SubmissionStatus::Accepted),
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
        }
    }

    pub fn from_record_patch(record: RecordPatch) -> Self {
        Self {
            hide_video: record.hide_video,
            is_verification: record.is_verification,
        }
    }
}

impl Submission {
    pub fn upsert_from_record_insert(
        conn: &mut DbConnection,
        record: RecordInsert,
    ) -> Result<Self, ApiError> {
        let existing_submission_id = submissions::table
            .filter(submissions::submitted_by.eq(record.submitted_by))
            .filter(submissions::level_id.eq(record.level_id))
            .select(submissions::id)
            .first::<Uuid>(conn)
            .optional()?;

        match existing_submission_id {
            Some(submission_id) => {
                let submission_update = SubmissionPatchMod::from_record_insert(record);
                return Ok(diesel::update(
                    submissions::table.filter(submissions::id.eq(submission_id)),
                )
                .set(&submission_update)
                .returning(Submission::as_select())
                .get_result::<Self>(conn)?);
            }
            None => {
                let submission_insert = SubmissionInsert::from_record_insert(record);
                return Ok(diesel::insert_into(submissions::table)
                    .values(&submission_insert)
                    .returning(Submission::as_select())
                    .get_result::<Self>(conn)?);
            }
        }
    }
}

impl Record {
    pub fn create(conn: &mut DbConnection, record: RecordInsert) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            // Create the corresponding submission first and let triggers initialize the record
            Submission::upsert_from_record_insert(conn, record.clone())?;

            // Then update the record-specific fields
            let record_patch = RecordUpdate::from_record_insert(record.clone());

            println!("Record patch: {:?}", record_patch);

            let result = diesel::update(records::table)
                .filter(records::submitted_by.eq(record.submitted_by))
                .filter(records::level_id.eq(record.level_id))
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
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| -> Result<Self, ApiError> {
            // Update the corresponding submission first and let triggers update the record
            let submission_patch = SubmissionPatchMod::from_record_update(record.clone());

            let (levelid, submitted_by): (Uuid, Uuid) = records::table
                .filter(records::id.eq(record_id))
                .select((records::level_id, records::submitted_by))
                .first(conn)?;

            diesel::update(submissions::table)
                .filter(submissions::submitted_by.eq(submitted_by))
                .filter(submissions::level_id.eq(levelid))
                .set(&submission_patch)
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

    pub fn delete(conn: &mut DbConnection, record_id: Uuid) -> Result<(), ApiError> {
        conn.transaction(|conn| -> Result<(), ApiError> {
            let (levelid, submitted_by): (Uuid, Uuid) = records::table
                .filter(records::id.eq(record_id))
                .select((records::level_id, records::submitted_by))
                .first(conn)?;

            diesel::update(submissions::table)
                .filter(submissions::submitted_by.eq(submitted_by))
                .filter(submissions::level_id.eq(levelid))
                .set((
                    submissions::status.eq(SubmissionStatus::Denied),
                    submissions::reviewer_notes
                        .eq(Some("Record deleted by a moderator".to_string())),
                ))
                .execute(conn)?;
            Ok(())
        })
    }
}

impl FullRecordUnresolved {
    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: RecordsQueryOptions,
    ) -> Result<Paginated<FullUnresolvedRecordPage>, ApiError> {
        let build_filtered = || {
            let mut q = records::table.into_boxed::<Pg>();
            if let Some(mobile) = options.mobile_filter {
                q = q.filter(records::mobile.eq(mobile));
            }
            if let Some(level) = options.level_filter {
                q = q.filter(records::level_id.eq(level));
            }
            if let Some(submitter) = options.submitter_filter {
                q = q.filter(records::submitted_by.eq(submitter));
            }
            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let raw_records = build_filtered()
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(FullRecordUnresolved::as_select())
            .load::<FullRecordUnresolved>(conn)?;

        let records: Vec<FullRecordUnresolvedDto> = raw_records
            .into_iter()
            .map(|record| FullRecordUnresolvedDto { record })
            .collect();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            FullUnresolvedRecordPage { data: records },
        ))
    }
}

impl FullRecordResolved {
    pub fn find(conn: &mut DbConnection, record_id: Uuid) -> Result<Self, ApiError> {
        let (record, user, level): (FullRecordTemplate<Uuid, Uuid>, BaseUser, ExtendedBaseLevel) =
            records::table
                .filter(records::id.eq(record_id))
                .inner_join(users::table.on(records::submitted_by.eq(users::id)))
                .inner_join(levels::table.on(records::level_id.eq(levels::id)))
                .order_by(records::completion_time.asc())
                .select((
                    FullRecordTemplate::<Uuid, Uuid>::as_select(),
                    BaseUser::as_select(),
                    ExtendedBaseLevel::as_select(),
                ))
                .first(conn)?;

        Ok(Self::from_data(record, user, level))
    }

    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: RecordsQueryOptions,
    ) -> Result<Paginated<FullResolvedRecordPage>, ApiError> {
        let build_filtered = || {
            let mut q = records::table.into_boxed::<Pg>();
            if let Some(mobile) = options.mobile_filter {
                q = q.filter(records::mobile.eq(mobile));
            }
            if let Some(level) = options.level_filter {
                q = q.filter(records::level_id.eq(level));
            }
            if let Some(submitter) = options.submitter_filter {
                q = q.filter(records::submitted_by.eq(submitter));
            }
            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let records = build_filtered()
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
            .order_by(records::completion_time.asc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                FullRecordUnresolved::as_select(),
                BaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(FullRecordUnresolved, BaseUser, ExtendedBaseLevel)>(conn)?;

        let records_resolved: Vec<Self> = records
            .into_iter()
            .map(|(record, user, level)| Self::from_data(record, user, level))
            .collect();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            FullResolvedRecordPage {
                data: records_resolved,
            },
        ))
    }

    fn from_data(record: FullRecordUnresolved, user: BaseUser, level: ExtendedBaseLevel) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            level_id: level,
            mobile: record.mobile,
            video_url: record.video_url,
            completion_time: record.completion_time,
            is_verification: record.is_verification,
            created_at: record.created_at,
            updated_at: record.updated_at,
            hide_video: record.hide_video,
        }
    }
}
