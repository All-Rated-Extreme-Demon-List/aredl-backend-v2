use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::packtiers::BasePackTier;
use crate::list::List;
use crate::schema::users;
use crate::schema::{completed_packs, packs};
use crate::users::ExtendedBaseUser;
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
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
    pub fn create(conn: &mut DbConnection, list: List, pack: PackCreate) -> Result<Self, ApiError> {
        let pack = diesel::insert_into(packs::table)
            .values((packs::list_id.eq(list), pack))
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(pack)
    }

    pub fn update(
        conn: &mut DbConnection,
        list: List,
        id: Uuid,
        pack: PackUpdate,
    ) -> Result<Self, ApiError> {
        let pack = diesel::update(packs::table)
            .filter(packs::list_id.eq(list))
            .filter(packs::id.eq(id))
            .set(pack)
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(pack)
    }

    pub fn delete(conn: &mut DbConnection, list: List, id: Uuid) -> Result<Self, ApiError> {
        let pack = diesel::delete(packs::table)
            .filter(packs::list_id.eq(list))
            .filter(packs::id.eq(id))
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(pack)
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CompletedPackVictor {
    /// User who completed the pack.
    pub user: ExtendedBaseUser,
    /// Timestamp when the pack was completed (highest achieved_at for each level record in the pack)
    pub completed_at: chrono::NaiveDateTime,
}

impl Pack {
    pub fn find_victors(
        conn: &mut DbConnection,
        list: List,
        pack_id: Uuid,
    ) -> Result<Vec<CompletedPackVictor>, ApiError> {
        let victors = completed_packs::table
            .filter(completed_packs::list_id.eq(list))
            .inner_join(users::table)
            .filter(completed_packs::pack_id.eq(pack_id))
            .select((completed_packs::completed_at, ExtendedBaseUser::as_select()))
            .order(completed_packs::completed_at.asc())
            .load::<(chrono::NaiveDateTime, ExtendedBaseUser)>(conn)?
            .into_iter()
            .map(|(completed_at, user)| CompletedPackVictor { user, completed_at })
            .collect::<Vec<CompletedPackVictor>>();

        Ok(victors)
    }
}
