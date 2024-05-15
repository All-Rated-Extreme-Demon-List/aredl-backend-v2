use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::prelude::*;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use crate::schema::aredl_levels;
use crate::db;
use crate::error_handler::CustomError;

#[derive(Serialize, Deserialize, Queryable)]
pub struct Level {
    pub id : Uuid,
    pub position: i32,
    pub name: String,
    pub points: i32,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
}

#[derive(Serialize, Deserialize, Insertable)]
#[diesel(table_name=aredl_levels)]
pub struct LevelPlace {
    pub position: i32,
    pub name: String,
    pub legacy: bool,
    pub level_id: i32,
    pub two_player: bool,
}

#[derive(Serialize, Deserialize, AsChangeset)]
#[diesel(table_name=aredl_levels)]
pub struct LevelUpdate {
    pub position: Option<i32>,
    pub name: Option<String>,
    pub legacy: Option<bool>,
    pub two_player: Option<bool>,
}

impl Level {
    pub fn find_all() -> Result<Vec<Self>, CustomError>{
        let conn = &mut db::connection()?;
        let levels = aredl_levels::table
            .order(aredl_levels::position)
            .load::<Level>(conn)?;
        Ok(levels)
    }

    pub fn find(id: Uuid) -> Result<Self, CustomError> {
        let conn = &mut db::connection()?;
        let level = aredl_levels::table
            .filter(aredl_levels::id.eq(id))
            .first(conn)?;
        Ok(level)
    }

    pub fn create(level: LevelPlace) -> Result<Self, CustomError> {
        let conn = &mut db::connection()?;
        let level = diesel::insert_into(aredl_levels::table)
            .values(level)
            .get_result(conn)?;
        Ok(level)
    }

    pub fn update(id: Uuid, level: LevelUpdate) -> Result<Self, CustomError> {
        let conn = &mut db::connection()?;
        let level = diesel::update(aredl_levels::table)
            .set(level)
            .filter(aredl_levels::id.eq(id))
            .get_result(conn)?;
        Ok(level)
    }
}