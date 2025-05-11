use crate::aredl::packs::{BasePack, PackWithTierResolved};
use crate::aredl::packtiers::BasePackTier;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::aredl::{pack_levels, pack_tiers, packs};
use actix_web::web;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use uuid::Uuid;

impl PackWithTierResolved {
    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let packs = packs::table
            .inner_join(pack_levels::table.on(pack_levels::pack_id.eq(packs::id)))
            .filter(pack_levels::level_id.eq(level_id))
            .inner_join(pack_tiers::table.on(packs::tier.eq(pack_tiers::id)))
            .select((BasePack::as_select(), BasePackTier::as_select()))
            .load::<(BasePack, BasePackTier)>(&mut db.connection()?)?;
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
