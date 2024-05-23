use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::schema::{aredl_levels, users};
use crate::db;
use crate::error_handler::ApiError;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_levels)]
pub struct Level {
    pub id : Uuid,
    pub position: i32,
    pub name: String,
    pub publisher_id: Uuid,
    pub points: i32,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=users)]
pub struct Publisher {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
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
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[diesel(table_name=aredl_levels)]
pub struct LevelUpdate {
    pub position: Option<i32>,
    pub name: Option<String>,
    pub publisher_id: Option<Uuid>,
    pub legacy: Option<bool>,
    pub two_player: Option<bool>,
}

impl Level {
    pub fn find_all() -> Result<Vec<Self>, ApiError>{
        let levels = aredl_levels::table
            .order(aredl_levels::position)
            .load::<Self>(&mut db::connection()?)?;
        Ok(levels)
    }

    pub fn find(id: Uuid) -> Result<Self, ApiError> {
        let level = aredl_levels::table
            .filter(aredl_levels::id.eq(id))
            .first(&mut db::connection()?)?;
        Ok(level)
    }

    pub fn find_resolved(id: Uuid) -> Result<(Self, Publisher), ApiError> {
        let level_resolved = aredl_levels::table
            .filter(aredl_levels::id.eq(id))
            .inner_join(users::table)
            .select((Level::as_select(), Publisher::as_select()))
            .first::<(Level, Publisher)>(&mut db::connection()?)?;
        Ok(level_resolved)
    }

    pub fn create(level: LevelPlace) -> Result<Self, ApiError> {
        let level = diesel::insert_into(aredl_levels::table)
            .values(level)
            .get_result(&mut db::connection()?)?;
        Ok(level)
    }

    pub fn update(id: Uuid, level: LevelUpdate) -> Result<Self, ApiError> {
        let level = diesel::update(aredl_levels::table)
            .set(level)
            .filter(aredl_levels::id.eq(id))
            .get_result(&mut db::connection()?)?;
        Ok(level)
    }
}