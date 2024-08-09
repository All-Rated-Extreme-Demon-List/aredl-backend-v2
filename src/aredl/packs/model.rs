use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::aredl_packs;

#[derive(Serialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
    pub tier: Uuid,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct PackCreate {
    pub name: String,
    pub tier: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct PackUpdate {
    pub name: Option<String>,
    pub tier: Option<Uuid>,
}

impl Pack {
    pub fn create(db: web::Data<Arc<DbAppState>>, pack: PackCreate) -> Result<Self, ApiError> {
        let pack = diesel::insert_into(aredl_packs::table)
            .values(pack)
            .get_result(&mut db.connection()?)?;
        Ok(pack)
    }

    pub fn update(db: web::Data<Arc<DbAppState>>, id: Uuid, pack: PackUpdate) -> Result<Self, ApiError> {
        let pack = diesel::update(aredl_packs::table)
            .filter(aredl_packs::id.eq(id))
            .set(pack)
            .get_result(&mut db.connection()?)?;
        Ok(pack)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Self, ApiError> {
        let pack = diesel::delete(aredl_packs::table)
            .filter(aredl_packs::id.eq(id))
            .get_result(&mut db.connection()?)?;
        Ok(pack)
    }
}