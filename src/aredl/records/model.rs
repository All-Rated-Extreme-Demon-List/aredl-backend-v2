use crate::aredl::levels::BaseLevel;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{aredl_levels, aredl_records, users};
use crate::users::BaseUser;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::query_dsl::JoinOnDsl;
use diesel::sql_types::Bool;
use diesel::{
    BoxableExpression, ExpressionMethods, Insertable, IntoSql, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
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
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Placement order of the record in the records list of this level.
    pub placement_order: i32,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
    /// Timestamp of when the record was last updated.
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
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
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Whether this record is the verification of this level or not.
    pub is_verification: bool,
    /// Placement order of the record in the records list of this level.
    #[serde(skip_serializing)]
    pub placement_order: i32,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
    /// Timestamp of when the record was last updated.
    pub updated_at: NaiveDateTime,
}

pub type FullRecordUnresolved = FullRecordTemplate<Uuid, Uuid>;
pub type FullRecordResolved = FullRecordTemplate<BaseLevel, BaseUser>;

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
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
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<NaiveDateTime>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
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
    /// Whether this record is the verification of this level or not.
    pub is_verification: Option<bool>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: Option<NaiveDateTime>,
    /// Timestamp of when the record was last updated.
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct RecordTemplate<T> {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: T,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
}

pub type RecordUnresolved = RecordTemplate<Uuid>;
pub type RecordResolved = RecordTemplate<BaseUser>;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecordsQueryOptions {
    pub mobile_filter: Option<bool>,
    pub level_filter: Option<Uuid>,
    pub submitter_filter: Option<Uuid>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct RecordPage {
    data: Vec<FullRecordResolved>,
}

impl Record {
    pub fn create(db: web::Data<Arc<DbAppState>>, record: RecordInsert) -> Result<Self, ApiError> {
        let record = diesel::insert_into(aredl_records::table)
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
        let record = diesel::update(aredl_records::table)
            .filter(aredl_records::id.eq(record_id))
            .set(record)
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, record_id: Uuid) -> Result<Self, ApiError> {
        let record = diesel::delete(aredl_records::table)
            .filter(aredl_records::id.eq(record_id))
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }
}

impl FullRecordResolved {
    pub fn find_all<const D: i64>(
        db: web::Data<Arc<DbAppState>>,
        page_query: PageQuery<D>,
        options: RecordsQueryOptions,
    ) -> Result<Paginated<RecordPage>, ApiError> {
        let conn = &mut db.connection()?;

        let total_count: i64 = aredl_records::table
            .filter(options.mobile_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |mobile| Box::new(aredl_records::mobile.eq(mobile)),
            ))
            .filter(options.level_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |level| Box::new(aredl_records::level_id.eq(level)),
            ))
            .filter(options.submitter_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |submitter| Box::new(aredl_records::submitted_by.eq(submitter)),
            ))
            .count()
            .get_result(conn)?;

        let query = aredl_records::table.into_boxed::<Pg>();
        let records = query
            .filter(options.mobile_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |mobile| Box::new(aredl_records::mobile.eq(mobile)),
            ))
            .filter(options.level_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |level| Box::new(aredl_records::level_id.eq(level)),
            ))
            .filter(options.submitter_filter.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |submitter| Box::new(aredl_records::submitted_by.eq(submitter)),
            ))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .inner_join(aredl_levels::table.on(aredl_records::level_id.eq(aredl_levels::id)))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                FullRecordUnresolved::as_select(),
                BaseUser::as_select(),
                BaseLevel::as_select(),
            ))
            .load::<(FullRecordUnresolved, BaseUser, BaseLevel)>(conn)?;

        let records_resolved: Vec<Self> = records
            .into_iter()
            .map(|(record, user, level)| Self::from_data(record, user, level))
            .collect();

        let page = RecordPage {
            data: records_resolved,
        };

        Ok(Paginated::from_data(page_query, total_count, page))
    }

    fn from_data(record: FullRecordUnresolved, user: BaseUser, level: BaseLevel) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            level_id: level,
            mobile: record.mobile,
            ldm_id: record.ldm_id,
            video_url: record.video_url,
            raw_url: record.raw_url,
            is_verification: record.is_verification,
            placement_order: record.placement_order,
            reviewer_id: record.reviewer_id,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}
