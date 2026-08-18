use std::collections::HashMap;

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    aredl::bounty::BountyType as ClassicBountyType,
    aredl::levels::LevelStatus as ClassicLevelStatus,
    arepl::bounty::BountyType as PlatformerBountyType,
    arepl::levels::LevelStatus as PlatformerLevelStatus,
    error_handler::ApiError,
    schema::{
        aredl::{
            self, badge_level_statistics as classic_badge_level_statistics,
            completed_packs as classic_completed_packs,
        },
        arepl::{
            self, badge_level_statistics as platformer_badge_level_statistics,
            completed_packs as platformer_completed_packs,
        },
    },
};

#[derive(Debug)]
pub struct UserStatistics {
    pub classic: UserListStatistics,
    pub platformer: UserListStatistics,
    pub global: UserListStatistics,
}

#[derive(Debug, Clone)]
pub struct UserListStatistics {
    pub levels_records: Vec<BadgeLevelStatistics>,
    pub created_levels: Vec<BadgeCreatedLevelStatistics>,
    pub packs: Vec<BadgePackStatistics>,
    pub level_tag_counts: HashMap<String, i64>,
    pub bounty_counts: HashMap<String, i64>,
    pub leaderboard_rank: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct BadgeLevelStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub position: Option<i32>,
    pub current_position: Option<i32>,
    pub level_id: i32,
    pub two_player: bool,
    pub publisher_id: Uuid,
    pub edel_enjoyment: Option<f64>,
    pub nlw_tier: Option<String>,
    pub tags: Vec<Option<String>>,
    pub is_verification: bool,
    pub achieved_at: DateTime<Utc>,
    pub is_first_victor: bool,
    pub is_fastest_time: bool,
}

#[derive(Debug, Clone)]
pub struct BadgeCreatedLevelStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub position: Option<i32>,
    pub publisher_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct BadgePackStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub tier_name: String,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = classic_badge_level_statistics)]
struct ClassicBadgeLevelStatistics {
    id: Uuid,
    name: String,
    position: Option<i32>,
    current_position: Option<i32>,
    level_id: i32,
    two_player: bool,
    publisher_id: Uuid,
    edel_enjoyment: Option<f64>,
    nlw_tier: Option<String>,
    tags: Vec<Option<String>>,
    is_verification: bool,
    achieved_at: DateTime<Utc>,
    is_first_victor: bool,
    is_fastest_time: bool,
}

impl From<ClassicBadgeLevelStatistics> for BadgeLevelStatistics {
    fn from(row: ClassicBadgeLevelStatistics) -> Self {
        Self {
            scope: "classic",
            id: row.id,
            name: row.name,
            position: row.position,
            current_position: row.current_position,
            level_id: row.level_id,
            two_player: row.two_player,
            publisher_id: row.publisher_id,
            edel_enjoyment: row.edel_enjoyment,
            nlw_tier: row.nlw_tier,
            tags: row.tags,
            is_verification: row.is_verification,
            achieved_at: row.achieved_at,
            is_first_victor: row.is_first_victor,
            is_fastest_time: row.is_fastest_time,
        }
    }
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = platformer_badge_level_statistics)]
struct PlatformerBadgeLevelStatistics {
    id: Uuid,
    name: String,
    position: Option<i32>,
    current_position: Option<i32>,
    level_id: i32,
    two_player: bool,
    publisher_id: Uuid,
    edel_enjoyment: Option<f64>,
    nlw_tier: Option<String>,
    tags: Vec<Option<String>>,
    is_verification: bool,
    achieved_at: DateTime<Utc>,
    is_first_victor: bool,
    is_fastest_time: bool,
}

impl From<PlatformerBadgeLevelStatistics> for BadgeLevelStatistics {
    fn from(row: PlatformerBadgeLevelStatistics) -> Self {
        Self {
            scope: "platformer",
            id: row.id,
            name: row.name,
            position: row.position,
            current_position: row.current_position,
            level_id: row.level_id,
            two_player: row.two_player,
            publisher_id: row.publisher_id,
            edel_enjoyment: row.edel_enjoyment,
            nlw_tier: row.nlw_tier,
            tags: row.tags,
            is_verification: row.is_verification,
            achieved_at: row.achieved_at,
            is_first_victor: row.is_first_victor,
            is_fastest_time: row.is_fastest_time,
        }
    }
}

impl UserStatistics {
    pub fn load(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let classic = UserListStatistics::load_classic(conn, user_id)?;
        let platformer = UserListStatistics::load_platformer(conn, user_id)?;

        Ok(Self {
            global: UserListStatistics::combine(&classic, &platformer),
            classic,
            platformer,
        })
    }
}

impl UserListStatistics {
    fn load_classic(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let leaderboard_rank = aredl::user_leaderboard::table
            .filter(aredl::user_leaderboard::user_id.eq(user_id))
            .select(aredl::user_leaderboard::rank)
            .first::<i32>(conn)
            .optional()?;

        let levels_records = classic_badge_level_statistics::table
            .filter(classic_badge_level_statistics::submitted_by.eq(user_id))
            .order((
                classic_badge_level_statistics::position.asc().nulls_last(),
                classic_badge_level_statistics::name.asc(),
            ))
            .select(ClassicBadgeLevelStatistics::as_select())
            .load::<ClassicBadgeLevelStatistics>(conn)?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        let packs = classic_completed_packs::table
            .inner_join(
                aredl::packs::table.on(aredl::packs::id.eq(classic_completed_packs::pack_id)),
            )
            .inner_join(aredl::pack_tiers::table.on(aredl::pack_tiers::id.eq(aredl::packs::tier)))
            .filter(classic_completed_packs::user_id.eq(user_id))
            .order(aredl::pack_tiers::placement.asc())
            .select((
                aredl::packs::id,
                aredl::packs::name,
                aredl::pack_tiers::name,
            ))
            .load::<(Uuid, String, String)>(conn)?
            .into_iter()
            .map(|(id, name, tier_name)| BadgePackStatistics {
                scope: "classic",
                id,
                name,
                tier_name,
            })
            .collect::<Vec<_>>();

        let mut created_levels = aredl::levels::table
            .inner_join(
                aredl::levels_created::table
                    .on(aredl::levels_created::level_id.eq(aredl::levels::id)),
            )
            .filter(aredl::levels_created::user_id.eq(user_id))
            .filter(aredl::levels::status.ne(ClassicLevelStatus::Removed))
            .order(aredl::levels::position.asc())
            .select((
                aredl::levels::id,
                aredl::levels::name,
                aredl::levels::position,
                aredl::levels::publisher_id,
            ))
            .distinct()
            .load::<(Uuid, String, Option<i32>, Uuid)>(conn)?
            .into_iter()
            .map(
                |(id, name, position, publisher_id)| BadgeCreatedLevelStatistics {
                    scope: "classic",
                    id,
                    name,
                    position,
                    publisher_id,
                },
            )
            .collect::<Vec<_>>();

        let published_levels = aredl::levels::table
            .filter(aredl::levels::publisher_id.eq(user_id))
            .filter(aredl::levels::status.ne(ClassicLevelStatus::Removed))
            .order(aredl::levels::position.asc())
            .select((
                aredl::levels::id,
                aredl::levels::name,
                aredl::levels::position,
                aredl::levels::publisher_id,
            ))
            .load::<(Uuid, String, Option<i32>, Uuid)>(conn)?
            .into_iter()
            .map(
                |(id, name, position, publisher_id)| BadgeCreatedLevelStatistics {
                    scope: "classic",
                    id,
                    name,
                    position,
                    publisher_id,
                },
            );

        created_levels.extend(published_levels);
        created_levels.sort_by(|left, right| {
            left.position
                .unwrap_or(i32::MAX)
                .cmp(&right.position.unwrap_or(i32::MAX))
        });
        created_levels.dedup_by_key(|level| (level.id, level.publisher_id));

        let level_tag_counts = Self::count_level_tags(&levels_records);
        let bounty_counts = Self::count_classic_bounties(conn, user_id)?;

        Ok(Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
            bounty_counts,
            leaderboard_rank,
        })
    }

    fn load_platformer(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let levels_records = platformer_badge_level_statistics::table
            .filter(platformer_badge_level_statistics::submitted_by.eq(user_id))
            .order((
                platformer_badge_level_statistics::position
                    .asc()
                    .nulls_last(),
                platformer_badge_level_statistics::name.asc(),
            ))
            .select(PlatformerBadgeLevelStatistics::as_select())
            .load::<PlatformerBadgeLevelStatistics>(conn)?
            .into_iter()
            .map(Into::into)
            .collect::<Vec<_>>();

        let packs = platformer_completed_packs::table
            .inner_join(
                arepl::packs::table.on(arepl::packs::id.eq(platformer_completed_packs::pack_id)),
            )
            .inner_join(arepl::pack_tiers::table.on(arepl::pack_tiers::id.eq(arepl::packs::tier)))
            .filter(platformer_completed_packs::user_id.eq(user_id))
            .order(arepl::pack_tiers::placement.asc())
            .select((
                arepl::packs::id,
                arepl::packs::name,
                arepl::pack_tiers::name,
            ))
            .load::<(Uuid, String, String)>(conn)?
            .into_iter()
            .map(|(id, name, tier_name)| BadgePackStatistics {
                scope: "platformer",
                id,
                name,
                tier_name,
            })
            .collect::<Vec<_>>();

        let mut created_levels = arepl::levels::table
            .inner_join(
                arepl::levels_created::table
                    .on(arepl::levels_created::level_id.eq(arepl::levels::id)),
            )
            .filter(arepl::levels_created::user_id.eq(user_id))
            .filter(arepl::levels::status.ne(PlatformerLevelStatus::Removed))
            .order(arepl::levels::position.asc())
            .select((
                arepl::levels::id,
                arepl::levels::name,
                arepl::levels::position,
                arepl::levels::publisher_id,
            ))
            .distinct()
            .load::<(Uuid, String, Option<i32>, Uuid)>(conn)?
            .into_iter()
            .map(
                |(id, name, position, publisher_id)| BadgeCreatedLevelStatistics {
                    scope: "platformer",
                    id,
                    name,
                    position,
                    publisher_id,
                },
            )
            .collect::<Vec<_>>();

        let published_levels = arepl::levels::table
            .filter(arepl::levels::publisher_id.eq(user_id))
            .filter(arepl::levels::status.ne(PlatformerLevelStatus::Removed))
            .order(arepl::levels::position.asc())
            .select((
                arepl::levels::id,
                arepl::levels::name,
                arepl::levels::position,
                arepl::levels::publisher_id,
            ))
            .load::<(Uuid, String, Option<i32>, Uuid)>(conn)?
            .into_iter()
            .map(
                |(id, name, position, publisher_id)| BadgeCreatedLevelStatistics {
                    scope: "platformer",
                    id,
                    name,
                    position,
                    publisher_id,
                },
            );

        created_levels.extend(published_levels);
        created_levels.sort_by(|left, right| {
            left.position
                .unwrap_or(i32::MAX)
                .cmp(&right.position.unwrap_or(i32::MAX))
        });
        created_levels.dedup_by_key(|level| (level.id, level.publisher_id));

        let level_tag_counts = Self::count_level_tags(&levels_records);
        let bounty_counts = Self::count_platformer_bounties(conn, user_id)?;

        Ok(Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
            bounty_counts,
            leaderboard_rank: None,
        })
    }

    fn count_classic_bounties(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<HashMap<String, i64>, ApiError> {
        let mut counts = HashMap::new();
        for (bounty_type, key) in [
            (ClassicBountyType::Bounty, "bounty"),
            (ClassicBountyType::Weekly, "weekly"),
            (ClassicBountyType::Monthly, "monthly"),
            (ClassicBountyType::Event, "event"),
        ] {
            let count = aredl::bounty_completed::table
                .inner_join(
                    aredl::bounties::table
                        .on(aredl::bounties::id.eq(aredl::bounty_completed::bounty_id)),
                )
                .filter(aredl::bounty_completed::user_id.eq(user_id))
                .filter(aredl::bounties::bounty_type.eq(bounty_type))
                .count()
                .get_result::<i64>(conn)?;
            counts.insert(key.to_owned(), count);
        }
        Ok(counts)
    }

    fn count_platformer_bounties(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<HashMap<String, i64>, ApiError> {
        let mut counts = HashMap::new();
        for (bounty_type, key) in [
            (PlatformerBountyType::Bounty, "bounty"),
            (PlatformerBountyType::Weekly, "weekly"),
            (PlatformerBountyType::Monthly, "monthly"),
            (PlatformerBountyType::Event, "event"),
        ] {
            let count = arepl::bounty_completed::table
                .inner_join(
                    arepl::bounties::table
                        .on(arepl::bounties::id.eq(arepl::bounty_completed::bounty_id)),
                )
                .filter(arepl::bounty_completed::user_id.eq(user_id))
                .filter(arepl::bounties::bounty_type.eq(bounty_type))
                .count()
                .get_result::<i64>(conn)?;
            counts.insert(key.to_owned(), count);
        }
        Ok(counts)
    }

    fn count_level_tags(levels_records: &[BadgeLevelStatistics]) -> HashMap<String, i64> {
        let mut level_tag_counts = HashMap::new();
        for level in levels_records {
            for tag in level.tags.iter().flatten() {
                *level_tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
        }
        level_tag_counts
    }

    fn combine(classic: &Self, platformer: &Self) -> Self {
        let mut levels_records = classic.levels_records.clone();
        levels_records.extend(platformer.levels_records.clone());
        levels_records.sort_by(|left, right| {
            left.position
                .unwrap_or(i32::MAX)
                .cmp(&right.position.unwrap_or(i32::MAX))
                .then(right.is_verification.cmp(&left.is_verification))
                .then(left.scope.cmp(right.scope))
                .then(left.name.cmp(&right.name))
        });
        levels_records.dedup_by_key(|level| (level.scope, level.id));

        let mut created_levels = classic.created_levels.clone();
        created_levels.extend(platformer.created_levels.clone());
        created_levels.sort_by(|left, right| {
            left.position
                .unwrap_or(i32::MAX)
                .cmp(&right.position.unwrap_or(i32::MAX))
                .then(left.scope.cmp(right.scope))
                .then(left.name.cmp(&right.name))
        });
        created_levels.dedup_by_key(|level| (level.scope, level.id));

        let mut packs = classic.packs.clone();
        packs.extend(platformer.packs.clone());

        let mut level_tag_counts = classic.level_tag_counts.clone();
        for (tag, count) in &platformer.level_tag_counts {
            *level_tag_counts.entry(tag.clone()).or_insert(0) += count;
        }

        let mut bounty_counts = classic.bounty_counts.clone();
        for (bounty_type, count) in &platformer.bounty_counts {
            *bounty_counts.entry(bounty_type.clone()).or_insert(0) += count;
        }

        Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
            bounty_counts,
            leaderboard_rank: None,
        }
    }
}
