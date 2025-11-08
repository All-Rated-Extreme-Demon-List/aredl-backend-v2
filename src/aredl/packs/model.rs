use crate::aredl::packtiers::BasePackTier;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::aredl::packs;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=packs, check_for_backend(Pg))]
pub struct BasePack {
    /// Internal UUID of the pack.
    pub id: Uuid,
    /// Name of the pack.
    pub name: String,
}

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=packs, check_for_backend(Pg))]
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
#[diesel(table_name=packs, check_for_backend(Pg))]
pub struct PackCreate {
    /// Name of the pack to create.
    pub name: String,
    /// Internal UUID of the tier to add the pack to.
    pub tier: Uuid,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name=packs, check_for_backend(Pg))]
pub struct PackUpdate {
    /// New name of the pack.
    pub name: Option<String>,
    /// New tier of the pack.
    pub tier: Option<Uuid>,
}

impl Pack {
    pub fn create(conn: &mut DbConnection, pack: PackCreate) -> Result<Self, ApiError> {
        let pack = diesel::insert_into(packs::table)
            .values(pack)
            .get_result(conn)?;
        Ok(pack)
    }

    pub fn update(conn: &mut DbConnection, id: Uuid, pack: PackUpdate) -> Result<Self, ApiError> {
        let pack = diesel::update(packs::table)
            .filter(packs::id.eq(id))
            .set(pack)
            .get_result(conn)?;
        Ok(pack)
    }

    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let pack = diesel::delete(packs::table)
            .filter(packs::id.eq(id))
            .get_result(conn)?;
        Ok(pack)
    }
}
