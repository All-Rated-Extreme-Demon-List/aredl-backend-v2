use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::packs::{BasePack, PackWithTierResolved};
use crate::list::packtiers::BasePackTier;
use crate::list::List;
use crate::schema::{pack_levels, pack_tiers, packs};
use uuid::Uuid;

use diesel::prelude::*;
impl PackWithTierResolved {
    pub fn find_all(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let packs = packs::table
            .filter(packs::list_id.eq(list))
            .inner_join(pack_levels::table)
            .filter(pack_levels::list_id.eq(list))
            .filter(pack_levels::level_id.eq(level_id))
            .inner_join(pack_tiers::table)
            .select((BasePack::as_select(), BasePackTier::as_select()))
            .load::<(BasePack, BasePackTier)>(conn)?;
        let resolved = packs
            .into_iter()
            .map(|(pack, pack_tier)| PackWithTierResolved {
                pack,
                tier: pack_tier,
            })
            .collect::<Vec<_>>();
        Ok(resolved)
    }
}
