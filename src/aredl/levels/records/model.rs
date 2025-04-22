use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, Insertable, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::schema::{aredl_records, users};

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

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct RecordInsert {
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
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

impl Record {
    pub fn create(db: web::Data<Arc<DbAppState>>, level_id: Uuid, record: RecordInsert) -> Result<Self, ApiError> {
        let record = diesel::insert_into(aredl_records::table)
            .values((record, aredl_records::level_id.eq(level_id)))
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn update(db: web::Data<Arc<DbAppState>>, level_id: Uuid, record_id: Uuid, record: RecordUpdate) -> Result<Self, ApiError> {
        let record = diesel::update(aredl_records::table)
            .set(record)
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, level_id: Uuid, record_id: Uuid) -> Result<Self, ApiError> {
        let record = diesel::delete(aredl_records::table)
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .returning(Record::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(record)
    }

    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .select(Record::as_select())
            .order(aredl_records::is_verification.desc())
            .then_order_by(aredl_records::placement_order.asc())
            .load::<Self>(&mut db.connection()?)?;
        Ok(records)
    }

    pub fn find(db: web::Data<Arc<DbAppState>>, level_id: Uuid, record_id: Uuid) -> Result<Self, ApiError> {
        let record = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .select(Record::as_select())
            .first::<Self>(&mut db.connection()?)?;
        Ok(record)
    }
}

impl RecordResolved {
    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::is_verification.eq(false))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .order(aredl_records::placement_order.asc())
            .select((RecordUnresolved::as_select(), BaseUser::as_select()))
            .load::<(RecordUnresolved, BaseUser)>(&mut db.connection()?)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(records_resolved)
    }

    fn from_data(record: RecordUnresolved, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
        }
    }
}