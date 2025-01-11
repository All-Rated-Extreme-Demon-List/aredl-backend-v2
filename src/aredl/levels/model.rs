use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::schema::{aredl_levels, aredl_records, users};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_levels)]
pub struct Level {
    pub id: Uuid,
    pub position: i32,
    pub name: String,
    pub publisher_id: Uuid,
    pub points: i32,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
    pub tags: Vec<Option<String>>,
    pub description: Option<String>,
    pub edel_enjoyment: Option<f64>,
    pub is_edel_pending: bool,
    pub gddl_tier: Option<f64>,
    pub nlw_tier: Option<String>
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name=aredl_levels)]
pub struct LevelPlace {
    pub position: i32,
    pub name: String,
    pub publisher_id: Uuid,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
    pub tags: Option<Vec<Option<String>>>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[diesel(table_name=aredl_levels)]
pub struct LevelUpdate {
    pub position: Option<i32>,
    pub name: Option<String>,
    pub publisher_id: Option<Uuid>,
    pub legacy: Option<bool>,
    pub two_player: Option<bool>,
    pub tags: Option<Vec<Option<String>>>,
    pub description: Option<String>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
}

pub trait RecordSubmitter {}

impl RecordSubmitter for Uuid {}
impl RecordSubmitter for User {}

#[derive(Serialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_records)]
pub struct Record<T>
where T : RecordSubmitter
{
    pub id: Uuid,
    pub submitted_by: T,
    pub mobile: bool,
    pub video_url: String,
}

// Level struct that has publisher and verification resolved
#[derive(Serialize, Debug)]
pub struct ResolvedLevel {
    pub id: Uuid,
    pub position: i32,
    pub name: String,
    pub points: i32,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
    pub tags: Vec<Option<String>>,
    pub description: Option<String>,
    pub edel_enjoyment: Option<f64>,
    pub is_edel_pending: bool,
    pub gddl_tier: Option<f64>,
    pub nlw_tier: Option<String>,
    pub publisher: User,
    pub verification: Option<Record<User>>,
}

impl Level {
    pub fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<Vec<Self>, ApiError>{
        let levels = aredl_levels::table
            .select(Level::as_select())
            .order(aredl_levels::position)
            .load::<Self>(&mut db.connection()?)?;
        Ok(levels)
    }

    pub fn create(db: web::Data<Arc<DbAppState>>, level: LevelPlace) -> Result<Self, ApiError> {
        let level = diesel::insert_into(aredl_levels::table)
            .values(level)
            .returning(Self::as_select())
            .get_result(&mut db.connection()?)?;
        Ok(level)
    }

    pub fn update(db: web::Data<Arc<DbAppState>>, id: Uuid, level: LevelUpdate) -> Result<Self, ApiError> {
        let level = diesel::update(aredl_levels::table)
            .set(level)
            .filter(aredl_levels::id.eq(id))
            .returning(Self::as_select())
            .get_result(&mut db.connection()?)?;
        Ok(level)
    }
}

impl ResolvedLevel {
    pub fn find(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Self, ApiError> {
        let (level, publisher) = aredl_levels::table
            .filter(aredl_levels::id.eq(id))
            .inner_join(users::table.on(aredl_levels::publisher_id.eq(users::id)))
            .select(
                (Level::as_select(), User::as_select())
            )
            .first::<(Level, User)>(&mut db.connection()?)?;

        let verification = aredl_records::table
            .filter(aredl_records::level_id.eq(id))
            .filter(aredl_records::placement_order.eq(0))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .select(
                (Record::<Uuid>::as_select(), User::as_select())
            )
            .first::<(Record<Uuid>, User)>(&mut db.connection()?)
            .optional()?;

        let verification = verification.map(
            |record| Record::from(record)
        );

        let resolved_level = Self::from_data(level, publisher, verification);
        Ok(resolved_level)
    }

    pub fn from_data(level: Level, publisher: User, verification: Option<Record<User>>) -> Self {
        Self {
            id: level.id,
            position: level.position,
            name: level.name,
            points: level.points,
            legacy: level.legacy,
            level_id: level.level_id,
            two_player: level.two_player,
            tags: level.tags,
            description: level.description,
            edel_enjoyment: level.edel_enjoyment,
            is_edel_pending: level.is_edel_pending,
            gddl_tier: level.gddl_tier,
            nlw_tier: level.nlw_tier,
            publisher,
            verification,
        }
    }
}

impl From<(Record<Uuid>, User)> for Record<User> {
    fn from((record, user): (Record<Uuid>, User)) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
        }
    }
}