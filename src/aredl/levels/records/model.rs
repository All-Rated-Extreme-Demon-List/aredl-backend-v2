use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, Insertable, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::{aredl_records, users};

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct Record {
    pub id: Uuid,
    pub level_id: Uuid,
    pub submitted_by: Uuid,
    pub mobile: bool,
    pub ldm_id: Option<i32>,
    pub video_url: String,
    pub raw_url: Option<String>,
    pub placement_order: i32,
    pub reviewer_id: Option<Uuid>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct RecordInsert {
    pub submitted_by: Uuid,
    pub mobile: bool,
    pub ldm_id: Option<i32>,
    pub video_url: String,
    pub raw_url: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct RecordUpdate {
    pub submitted_by: Option<Uuid>,
    pub mobile: Option<bool>,
    pub ldm_id: Option<i32>,
    pub video_url: Option<String>,
    pub raw_url: Option<String>,
    pub reviewer_id: Option<Uuid>,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_records, check_for_backend(Pg))]
pub struct RecordTemplate<T> {
    pub id: Uuid,
    pub submitted_by: T,
    pub mobile: bool,
    pub video_url: String,
    pub created_at: NaiveDateTime,
}

pub type RecordUnresolved = RecordTemplate<Uuid>;
pub type RecordResolved = RecordTemplate<User>;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
}

impl Record {
    pub fn create(level_id: Uuid, record: RecordInsert) -> Result<Self, ApiError> {
        let record = diesel::insert_into(aredl_records::table)
            .values((record, aredl_records::level_id.eq(level_id)))
            .get_result::<Self>(&mut db::connection()?)?;
        Ok(record)
    }

    pub fn update(level_id: Uuid, record_id: Uuid, record: RecordUpdate) -> Result<Self, ApiError> {
        let record = diesel::update(aredl_records::table)
            .set(record)
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .get_result::<Self>(&mut db::connection()?)?;
        Ok(record)
    }

    pub fn delete(level_id: Uuid, record_id: Uuid) -> Result<Self, ApiError> {
        let record = diesel::delete(aredl_records::table)
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .get_result::<Self>(&mut db::connection()?)?;
        Ok(record)
    }

    pub fn find_all(level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .select(Record::as_select())
            .order(aredl_records::placement_order)
            .load::<Self>(&mut db::connection()?)?;
        Ok(records)
    }

    pub fn find(level_id: Uuid, record_id: Uuid) -> Result<Self, ApiError> {
        let record = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .first::<Self>(&mut db::connection()?)?;
        Ok(record)
    }
}

impl RecordResolved {
    pub fn find_all(level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::placement_order.ne(0))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .order(aredl_records::placement_order)
            .select((RecordUnresolved::as_select(), User::as_select()))
            .load::<(RecordUnresolved, User)>(&mut db::connection()?)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(records_resolved)
    }

    fn from_data(record: RecordUnresolved, user: User) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
        }
    }
}