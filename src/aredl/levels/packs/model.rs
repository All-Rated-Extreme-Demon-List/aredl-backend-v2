use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_levels, aredl_pack_tiers, aredl_packs};

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_packs)]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_pack_tiers)]
pub struct PackTier {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackResolved {
    pub id: Uuid,
    pub name: String,
    pub tier: PackTier,
}

impl PackResolved {

    pub fn find_all(level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let packs = aredl_packs::table
            .inner_join(aredl_pack_levels::table.on(aredl_pack_levels::pack_id.eq(aredl_packs::id)))
            .filter(aredl_pack_levels::level_id.eq(level_id))
            .inner_join(aredl_pack_tiers::table.on(aredl_packs::tier.eq(aredl_pack_tiers::id)))
            .select((Pack::as_select(), PackTier::as_select()))
            .load::<(Pack, PackTier)>(&mut db::connection()?)?;
        let resolved = packs.into_iter()
            .map(|(pack, pack_tier)| PackResolved {
                id: pack.id,
                name: pack.name,
                tier: pack_tier,
            }).collect::<Vec<_>>();
        Ok(resolved)
    }
}