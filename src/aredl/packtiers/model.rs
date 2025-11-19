use crate::app_data::db::DbConnection;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::error_handler::ApiError;
use crate::schema::{
    aredl::levels, aredl::pack_levels, aredl::pack_tiers, aredl::packs_points, aredl::records,
};
use diesel::pg::Pg;
use diesel::{
    BelongingToDsl, BoolExpressionMethods, ExpressionMethods, GroupedBy, JoinOnDsl,
    NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Identifiable, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=pack_tiers, check_for_backend(Pg))]
pub struct BasePackTier {
    /// Internal UUID of the pack tier.
    pub id: Uuid,
    /// Name of the pack tier.
    pub name: String,
    /// Color of the pack tier.
    pub color: String,
}

#[derive(Serialize, Identifiable, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=pack_tiers, check_for_backend(Pg))]
pub struct PackTier {
    /// Internal UUID of the pack tier.
    pub id: Uuid,
    /// Name of the pack tier.
    pub name: String,
    /// Color of the pack tier.
    pub color: String,
    /// Placement order in which the pack tier is displayed.
    pub placement: i32,
}

#[derive(Serialize, Identifiable, Associations, Selectable, Queryable, Debug, ToSchema)]
#[diesel(belongs_to(PackTier, foreign_key=tier))]
#[diesel(table_name=packs_points, check_for_backend(Pg))]
pub struct PackWithTier {
    /// Internal UUID of the pack.
    pub id: Uuid,
    /// Name of the pack.
    pub name: String,
    /// Internal UUID of the tier the pack belongs to.
    pub tier: Uuid,
    /// Points awarded for completing the pack.
    pub points: i32,
}

#[derive(Serialize, ToSchema)]
pub struct PackLevelResolved {
    #[serde(flatten)]
    pub pack_level: ExtendedBaseLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_by_user: Option<bool>,
}

#[derive(Serialize, ToSchema)]
pub struct PackWithLevelsResolved {
    /// Internal UUID of the pack.
    pub id: Uuid,
    /// Name of the pack.
    pub name: String,
    /// Points awarded for completing the pack.
    pub points: i32,
    /// Levels in the pack.
    pub levels: Vec<PackLevelResolved>,
}

#[derive(Serialize, ToSchema)]
pub struct PackTierResolved {
    /// Internal UUID of the pack tier.
    pub id: Uuid,
    /// Name of the pack tier.
    pub name: String,
    /// Color of the pack tier.
    pub color: String,
    /// Placement order in which the pack tier is displayed.
    pub placement: i32,
    /// Packs that belong to this pack tier.
    pub packs: Vec<PackWithLevelsResolved>,
}

#[derive(Serialize, Deserialize, Insertable, Debug, ToSchema)]
#[diesel(table_name=pack_tiers, check_for_backend(Pg))]
pub struct PackTierCreate {
    /// Name of the pack tier to create.
    pub name: String,
    /// Color of the pack tier to create.
    pub color: String,
    /// Placement order of the pack tier to create.
    pub placement: i32,
}

#[derive(Serialize, Deserialize, AsChangeset, Debug, ToSchema)]
#[diesel(table_name=pack_tiers, check_for_backend(Pg))]
pub struct PackTierUpdate {
    /// New name of the pack tier.
    pub name: Option<String>,
    /// New color of the pack tier.
    pub color: Option<String>,
    /// New placement order of the pack tier.
    pub placement: Option<i32>,
}

impl PackTierResolved {
    pub fn find_all(conn: &mut DbConnection, user_id: Option<Uuid>) -> Result<Vec<Self>, ApiError> {
        let pack_tiers = pack_tiers::table
            .order(pack_tiers::placement)
            .select(PackTier::as_select())
            .load::<PackTier>(conn)?;
        let packs = PackWithTier::belonging_to(&pack_tiers)
            .load::<PackWithTier>(conn)?
            .grouped_by(&pack_tiers);

        let levels_base_query =
            pack_levels::table.inner_join(levels::table.on(levels::id.eq(pack_levels::level_id)));

        let pack_levels = match user_id {
            Some(user) =>
            // join records and check if user has a record on the level.
            // any column can be used
            {
                levels_base_query
                    .left_join(
                        records::table.on(pack_levels::level_id
                            .eq(records::level_id)
                            .and(records::submitted_by.eq(user))),
                    )
                    .select((
                        pack_levels::pack_id,
                        ExtendedBaseLevel::as_select(),
                        records::id.nullable(),
                    ))
                    .load::<(Uuid, ExtendedBaseLevel, Option<Uuid>)>(conn)?
                    // map Option<Uuid> into Option<bool> which is always Some.
                    // It will be Some(true) if user has completed the level and Some(false) otherwise.
                    // That's because None is used for non-authenticated queries.
                    .into_iter()
                    .map(|(uuid, pack_level, completed)| {
                        (uuid, pack_level, Some(completed.is_some()))
                    })
                    .collect::<Vec<_>>()
            }
            None => levels_base_query
                .select((pack_levels::pack_id, ExtendedBaseLevel::as_select()))
                .load::<(Uuid, ExtendedBaseLevel)>(conn)?
                // map to add None to signify that the completed_by_user field is missing
                .into_iter()
                .map(|(uuid, pack_level)| (uuid, pack_level, None))
                .collect::<Vec<_>>(),
        };

        let mut pack_levels_map: HashMap<Uuid, Vec<PackLevelResolved>> = HashMap::new();

        for (uuid, pack_level, completed_by_user) in pack_levels {
            pack_levels_map
                .entry(uuid)
                .or_insert_with(Vec::new)
                .push(PackLevelResolved {
                    pack_level,
                    completed_by_user,
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
                    .map(|pack| PackWithLevelsResolved {
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
    pub fn create(conn: &mut DbConnection, pack_tier: PackTierCreate) -> Result<Self, ApiError> {
        let tier = diesel::insert_into(pack_tiers::table)
            .values(pack_tier)
            .get_result(conn)?;
        Ok(tier)
    }

    pub fn update(
        conn: &mut DbConnection,
        id: Uuid,
        pack_tier: PackTierUpdate,
    ) -> Result<Self, ApiError> {
        let tier = diesel::update(pack_tiers::table)
            .set(pack_tier)
            .filter(pack_tiers::id.eq(id))
            .get_result(conn)?;
        Ok(tier)
    }

    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let tier = diesel::delete(pack_tiers::table)
            .filter(pack_tiers::id.eq(id))
            .get_result(conn)?;
        Ok(tier)
    }
}
