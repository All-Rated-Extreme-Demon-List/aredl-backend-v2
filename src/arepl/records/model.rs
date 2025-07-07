use crate::arepl::levels::ExtendedBaseLevel;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{arepl::levels, arepl::records, users};
use crate::users::BaseUser;
use actix_web::web;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::query_dsl::JoinOnDsl;
use diesel::{
    ExpressionMethods, Insertable, NullableExpressionMethods, PgExpressionMethods, QueryDsl,
    RunQueryDsl, Selectable, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
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
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Placement order of the record in the records list of this level.
    pub placement_order: i32,
    /// Name of the mod menu used for this record, if any.
    pub mod_menu: Option<String>,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Notes set by the reviewer when they accepted the record.
    pub reviewer_notes: Option<String>,
    /// Notes given by the user when they submitted the record.
    pub user_notes: Option<String>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct RecordInsert {
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Internal UUID of the level the record is for.
    pub level_id: Uuid,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name=records, check_for_backend(Pg))]
pub struct RecordUpdate {
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Option<Uuid>,
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Internal UUID of the level the record is for.
    pub level_id: Option<Uuid>,
    /// Completion time of the record in milliseconds.
    pub completion_time: Option<i64>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<DateTime<Utc>>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<DateTime<Utc>>,
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
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
}

pub type PublicRecordUnresolved = PublicRecordTemplate<Uuid>;
pub type PublicRecordResolved = PublicRecordTemplate<BaseUser>;

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
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Name of the mod menu used for this record, if any.
    pub mod_menu: Option<String>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Placement order of the record in the records list of this level.
    #[serde(skip_serializing)]
    pub placement_order: i32,
    /// Internal UUID of the user who reviewed the record.
    #[serde(rename = "reviewer")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer_id: Option<UserT>,
    /// Notes set by the reviewer when they accepted the record.
    pub reviewer_notes: Option<String>,
    /// Notes given by the user when they submitted the record.
    pub user_notes: Option<String>,
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
    pub reviewer_filter: Option<Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FullUnresolvedRecordPage {
    data: Vec<FullRecordUnresolvedDto>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct FullResolvedRecordPage {
    data: Vec<FullRecordResolved>,
}

impl Record {
    pub fn create(db: web::Data<Arc<DbAppState>>, record: RecordInsert) -> Result<Self, ApiError> {
        let record = diesel::insert_into(records::table)
            .values(record)
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn update(
        db: web::Data<Arc<DbAppState>>,
        record_id: Uuid,
        record: RecordUpdate,
    ) -> Result<Self, ApiError> {
        let record = diesel::update(records::table)
            .filter(records::id.eq(record_id))
            .set(record)
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, record_id: Uuid) -> Result<Self, ApiError> {
        let record = diesel::delete(records::table)
            .filter(records::id.eq(record_id))
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }
}

impl FullRecordUnresolved {
    pub fn find_all<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        mut options: RecordsQueryOptions,
        hide_reviewer: bool,
    ) -> Result<Paginated<FullUnresolvedRecordPage>, ApiError> {
        let conn = &mut db.connection()?;

        if hide_reviewer {
            options.reviewer_filter = None;
        }

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
            if let Some(reviewer) = options.reviewer_filter {
                q = q.filter(records::reviewer_id.is_not_distinct_from(reviewer));
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
            .map(|record| FullRecordUnresolvedDto {
                record: {
                    if hide_reviewer {
                        Self {
                            reviewer_id: None,
                            ..record
                        }
                    } else {
                        record
                    }
                },
            })
            .collect();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            FullUnresolvedRecordPage { data: records },
        ))
    }
}

impl FullRecordResolved {
    pub fn find(db: web::Data<Arc<DbAppState>>, record_id: Uuid) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let reviewers = alias!(users as reviewers);

        let (record, user, level, reviewer): (
            FullRecordTemplate<Uuid, Uuid>,
            BaseUser,
            ExtendedBaseLevel,
            Option<BaseUser>,
        ) = records::table
            .filter(records::id.eq(record_id))
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
            .order_by(records::completion_time.asc())
            .left_join(
                reviewers.on(reviewers
                    .field(users::id)
                    .nullable()
                    .eq(records::reviewer_id.nullable())),
            )
            .select((
                FullRecordTemplate::<Uuid, Uuid>::as_select(),
                BaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
                reviewers
                    .fields(<BaseUser as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .first(conn)?;

        Ok(Self::from_data(record, user, level, reviewer))
    }

    pub fn find_all<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        mut options: RecordsQueryOptions,
        hide_reviewer: bool,
    ) -> Result<Paginated<FullResolvedRecordPage>, ApiError> {
        let conn = &mut db.connection()?;

        if hide_reviewer {
            options.reviewer_filter = None;
        }

        let reviewers = alias!(users as reviewers);

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
            if let Some(reviewer) = options.reviewer_filter {
                q = q.filter(records::reviewer_id.is_not_distinct_from(reviewer));
            }
            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let records = build_filtered()
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .inner_join(levels::table.on(records::level_id.eq(levels::id)))
            .left_join(
                reviewers.on(reviewers
                    .field(users::id)
                    .nullable()
                    .eq(records::reviewer_id.nullable())),
            )
            .order_by(records::completion_time.asc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                FullRecordUnresolved::as_select(),
                BaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
                reviewers
                    .fields(<BaseUser as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .load::<(
                FullRecordUnresolved,
                BaseUser,
                ExtendedBaseLevel,
                Option<BaseUser>,
            )>(conn)?;

        let mut records_resolved: Vec<Self> = records
            .into_iter()
            .map(|(record, user, level, reviewer)| Self::from_data(record, user, level, reviewer))
            .collect();

        if hide_reviewer {
            for record in records_resolved.iter_mut() {
                record.reviewer_id = None;
            }
        }

        Ok(Paginated::from_data(
            page_query,
            total_count,
            FullResolvedRecordPage {
                data: records_resolved,
            },
        ))
    }

    fn from_data(
        record: FullRecordUnresolved,
        user: BaseUser,
        level: ExtendedBaseLevel,
        reviewer: Option<BaseUser>,
    ) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            level_id: level,
            mobile: record.mobile,
            ldm_id: record.ldm_id,
            video_url: record.video_url,
            raw_url: record.raw_url,
            completion_time: record.completion_time,
            is_verification: record.is_verification,
            placement_order: record.placement_order,
            reviewer_id: reviewer,
            reviewer_notes: record.reviewer_notes,
            user_notes: record.user_notes,
            mod_menu: record.mod_menu,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
