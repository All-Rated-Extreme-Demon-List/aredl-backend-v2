use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{BelongingToDsl, GroupedBy, QueryDsl, RunQueryDsl, SelectableHelper};
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_tiers, aredl_packs};

#[derive(Serialize, Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_pack_tiers, check_for_backend(Pg))]
pub struct PackTier {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub placement: i32,
}

#[derive(Serialize, Identifiable, Associations, Selectable, Queryable, Debug)]
#[diesel(belongs_to(PackTier, foreign_key=tier))]
#[diesel(table_name=aredl_packs, check_for_backend(Pg))]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
    pub tier: Uuid,
}

#[derive(Serialize)]
pub struct PackTierResolved {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub placement: i32,
    pub packs: Vec<Pack>,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_pack_tiers, check_for_backend(Pg))]
pub struct PackTierCreate {
    pub name: String,
    pub color: String,
    pub placement: i32,
}

impl PackTierResolved {
    pub fn find_all() -> Result<Vec<Self>, ApiError> {
        let pack_tiers = aredl_pack_tiers::table
            .order(aredl_pack_tiers::placement)
            .select(PackTier::as_select())
            .load::<PackTier>(&mut db::connection()?)?;
        let packs = Pack::belonging_to(&pack_tiers)
            .load::<Pack>(&mut db::connection()?)?
            .grouped_by(&pack_tiers);
        let resolved = pack_tiers
            .into_iter()
            .zip(packs)
            .map(|(tier, packs)| PackTierResolved {
                id: tier.id,
                name: tier.name,
                color: tier.color,
                placement: tier.placement,
                packs,
            })
            .collect::<Vec<_>>();
        Ok(resolved)
    }
}

impl PackTier {
    pub fn create(pack_tier: PackTierCreate) -> Result<Self, ApiError> {
        let tier = diesel::insert_into(aredl_pack_tiers::table)
            .values(pack_tier)
            .get_result(&mut db::connection()?)?;
        Ok(tier)
    }
}
