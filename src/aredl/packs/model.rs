use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::aredl_packs;
use crate::aredl::packtiers::BasePackTier;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct BasePack {
    /// Internal UUID of the pack.
    pub id: Uuid,
    /// Name of the pack.
    pub name: String,
}

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct Pack {
    /// Internal UUID of the pack.
    pub id: Uuid,
    /// Name of the pack.
    pub name: String,
    /// Internal UUID of the tier the pack belongs to.
    pub tier: Uuid,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct PackWithTierResolved {
    #[serde(flatten)]
    pub pack: BasePack,
    /// Tier the pack belongs to.
    pub tier: BasePackTier,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct PackCreate {
    /// Name of the pack to create.
    pub name: String,
    /// Internal UUID of the tier to add the pack to.
    pub tier: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct PackUpdate {
    /// New name of the pack.
    pub name: Option<String>,
    /// New tier of the pack.
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