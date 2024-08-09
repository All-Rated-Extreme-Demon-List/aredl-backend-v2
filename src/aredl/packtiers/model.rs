use std::collections::HashMap;
use std::sync::Arc;
use actix_web::web;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{BelongingToDsl, BoolExpressionMethods, ExpressionMethods, GroupedBy, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_tiers, aredl_pack_levels, aredl_levels, aredl_records};
use crate::custom_schema::aredl_packs_points;
use crate::db::DbAppState;

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
#[diesel(table_name=aredl_packs_points, check_for_backend(Pg))]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
    pub tier: Uuid,
    pub points: i32,
}

#[derive(Serialize, Identifiable, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_levels, check_for_backend(Pg))]
pub struct PackLevel {
    pub id: Uuid,
    pub position: i32,
    pub name: String,
    pub points: i32,
    pub level_id: i32,
    pub two_player: bool,
}

#[derive(Serialize)]
pub struct PackLevelResolved {
    #[serde(flatten)]
    pub pack_level: PackLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_by_user: Option<bool>,
}

#[derive(Serialize)]
pub struct PackResolved {
    pub id: Uuid,
    pub name: String,
    pub points: i32,
    pub levels: Vec<PackLevelResolved>,
}

#[derive(Serialize)]
pub struct PackTierResolved {
    pub id: Uuid,
    pub name: String,
    pub color: String,
    pub placement: i32,
    pub packs: Vec<PackResolved>,
}

#[derive(Serialize, Deserialize, Insertable, Debug)]
#[diesel(table_name=aredl_pack_tiers, check_for_backend(Pg))]
pub struct PackTierCreate {
    pub name: String,
    pub color: String,
    pub placement: i32,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug)]
#[diesel(table_name=aredl_pack_tiers, check_for_backend(Pg))]
pub struct PackTierUpdate {
    pub name: Option<String>,
    pub color: Option<String>,
    pub placement: Option<i32>,
}

impl PackTierResolved {
    pub fn find_all(db: web::Data<Arc<DbAppState>>, user_id: Option<Uuid>) -> Result<Vec<Self>, ApiError> {
        let connection = &mut db.connection()?;

        let pack_tiers = aredl_pack_tiers::table
            .order(aredl_pack_tiers::placement)
            .select(PackTier::as_select())
            .load::<PackTier>(connection)?;
        let packs = Pack::belonging_to(&pack_tiers)
            .load::<Pack>(connection)?
            .grouped_by(&pack_tiers);

        let levels_base_query =
            aredl_pack_levels::table
                .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_pack_levels::level_id)));

        let pack_levels =
            match user_id {
                Some(user) =>
                    // join records and check if user has a record on the level.
                    // any column can be used, in this case we use aredl_records::placement_order because it is just an int.
                    levels_base_query
                        .left_join(aredl_records::table.on(
                            aredl_pack_levels::level_id.eq(aredl_records::level_id).and(
                            aredl_records::submitted_by.eq(user))))
                        .select((aredl_pack_levels::pack_id, PackLevel::as_select(), aredl_records::placement_order.nullable()))
                        .load::<(Uuid, PackLevel, Option<i32>)>(connection)?
                        // map Option<i32> into Option<bool> which is always Some.
                        // It will be Some(true) if user has completed the level and Some(false) otherwise.
                        // That's because None is used for non-authenticated queries.
                        .into_iter()
                        .map(|(uuid, pack_level, completed)|
                            (uuid, pack_level, Some(completed.is_some())))
                        .collect::<Vec<_>>(),
                None =>
                    levels_base_query
                        .select((aredl_pack_levels::pack_id, PackLevel::as_select()))
                        .load::<(Uuid, PackLevel)>(connection)?
                        // map to add None to signify that the completed_by_user field is missing
                        .into_iter()
                        .map(|(uuid, pack_level)| (uuid, pack_level, None))
                        .collect::<Vec<_>>(),
            };

        let mut pack_levels_map: HashMap<Uuid, Vec<PackLevelResolved>> = HashMap::new();

        for (uuid, pack_level, completed_by_user) in pack_levels {
            pack_levels_map.entry(uuid).or_insert_with(Vec::new).push(PackLevelResolved {
                pack_level,
                completed_by_user
            });
        }

        let resolved = pack_tiers
            .into_iter()
            .zip(packs)
            .map(|(tier, packs)| PackTierResolved {
                id: tier.id,
                name: tier.name,
                color: tier.color,
                placement: tier.placement,
                packs: packs
                    .into_iter()
                    .map(|pack|  PackResolved {
                        id: pack.id,
                        name: pack.name,
                        points: pack.points,
                        levels: pack_levels_map.remove(&pack.id).unwrap_or_else(Vec::new),
                    })
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        Ok(resolved)
    }
}

impl PackTier {
    pub fn create(db: web::Data<Arc<DbAppState>>, pack_tier: PackTierCreate) -> Result<Self, ApiError> {
        let tier = diesel::insert_into(aredl_pack_tiers::table)
            .values(pack_tier)
            .get_result(&mut db.connection()?)?;
        Ok(tier)
    }

    pub fn update(db: web::Data<Arc<DbAppState>>, id: Uuid, pack_tier: PackTierUpdate) -> Result<Self, ApiError> {
        let tier = diesel::update(aredl_pack_tiers::table)
            .set(pack_tier)
            .filter(aredl_pack_tiers::id.eq(id))
            .get_result(&mut db.connection()?)?;
        Ok(tier)
    }

    pub fn delete(db: web::Data<Arc<DbAppState>>, id: Uuid) -> Result<Self, ApiError> {
        let tier = diesel::delete(aredl_pack_tiers::table)
            .filter(aredl_pack_tiers::id.eq(id))
            .get_result(&mut db.connection()?)?;
        Ok(tier)
    }
}
