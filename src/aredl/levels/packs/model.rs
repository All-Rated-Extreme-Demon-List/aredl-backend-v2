use std::sync::Arc;
use actix_web::web;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use crate::db::DbAppState;
use crate::aredl::packs::{BasePack, PackWithTierResolved};
use crate::aredl::packtiers::BasePackTier;
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_levels, aredl_pack_tiers, aredl_packs};

impl PackWithTierResolved {

    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let packs = aredl_packs::table
            .inner_join(aredl_pack_levels::table.on(aredl_pack_levels::pack_id.eq(aredl_packs::id)))
            .filter(aredl_pack_levels::level_id.eq(level_id))
            .inner_join(aredl_pack_tiers::table.on(aredl_packs::tier.eq(aredl_pack_tiers::id)))
            .select((BasePack::as_select(), BasePackTier::as_select()))
            .load::<(BasePack, BasePackTier)>(&mut db.connection()?)?;
        let resolved = packs.into_iter()
            .map(|(pack, pack_tier)| PackWithTierResolved {
                pack,
                tier: pack_tier,
            }).collect::<Vec<_>>();
        Ok(resolved)
    }
}