use std::collections::{HashMap, HashSet};

use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    schema::{
        aredl::{self, completed_packs as classic_completed_packs},
        arepl::{self, completed_packs as platformer_completed_packs},
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
}

#[derive(Debug, Clone)]
pub struct BadgeLevelStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub publisher_id: Uuid,
    pub is_verification: bool,
    pub is_first_victor: bool,
}

#[derive(Debug, Clone)]
pub struct BadgeCreatedLevelStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub position: i32,
    pub publisher_id: Uuid,
}

#[derive(Debug, Clone)]
pub struct BadgePackStatistics {
    pub scope: &'static str,
    pub id: Uuid,
    pub name: String,
    pub tier_name: String,
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
        let first_victor_level_ids = aredl::records::table
            .inner_join(aredl::levels::table.on(aredl::levels::id.eq(aredl::records::level_id)))
            .filter(aredl::levels::legacy.eq(false))
            .filter(aredl::records::is_verification.eq(false))
            .distinct_on(aredl::records::level_id)
            .order_by((
                aredl::records::level_id.asc(),
                aredl::records::achieved_at.asc(),
                aredl::records::created_at.asc(),
                aredl::records::id.asc(),
            ))
            .select((aredl::records::level_id, aredl::records::submitted_by))
            .load::<(Uuid, Uuid)>(conn)?
            .into_iter()
            .filter_map(|(level_id, submitted_by)| (submitted_by == user_id).then_some(level_id))
            .collect::<HashSet<_>>();

        let mut levels_records = HashMap::new();
        for (id, name, position, publisher_id, is_verification) in aredl::records::table
            .inner_join(aredl::levels::table.on(aredl::levels::id.eq(aredl::records::level_id)))
            .filter(aredl::records::submitted_by.eq(user_id))
            .filter(aredl::levels::legacy.eq(false))
            .order(aredl::levels::position.asc())
            .select((
                aredl::levels::id,
                aredl::levels::name,
                aredl::levels::position,
                aredl::levels::publisher_id,
                aredl::records::is_verification,
            ))
            .load::<(Uuid, String, i32, Uuid, bool)>(conn)?
        {
            levels_records
                .entry(id)
                .and_modify(|level: &mut BadgeLevelStatistics| {
                    level.is_verification |= is_verification;
                })
                .or_insert(BadgeLevelStatistics {
                    scope: "classic",
                    id,
                    name,
                    position,
                    publisher_id,
                    is_verification,
                    is_first_victor: first_victor_level_ids.contains(&id),
                });
        }

        let mut levels_records = levels_records.into_values().collect::<Vec<_>>();
        levels_records.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(left.name.cmp(&right.name))
        });

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
            .order(aredl::levels::position.asc())
            .select((
                aredl::levels::id,
                aredl::levels::name,
                aredl::levels::position,
                aredl::levels::publisher_id,
            ))
            .distinct()
            .load::<(Uuid, String, i32, Uuid)>(conn)?
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
            .order(aredl::levels::position.asc())
            .select((
                aredl::levels::id,
                aredl::levels::name,
                aredl::levels::position,
                aredl::levels::publisher_id,
            ))
            .load::<(Uuid, String, i32, Uuid)>(conn)?
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
        created_levels.sort_by_key(|level| level.position);
        created_levels.dedup_by_key(|level| (level.id, level.publisher_id));

        let completed_level_tags = aredl::records::table
            .inner_join(aredl::levels::table.on(aredl::levels::id.eq(aredl::records::level_id)))
            .filter(aredl::records::submitted_by.eq(user_id))
            .filter(aredl::levels::legacy.eq(false))
            .select((aredl::records::level_id, aredl::levels::tags))
            .distinct()
            .load::<(Uuid, Vec<Option<String>>)>(conn)?;

        let level_tag_counts = Self::count_level_tags(completed_level_tags);

        Ok(Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
        })
    }

    fn load_platformer(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let first_victor_level_ids = arepl::records::table
            .inner_join(arepl::levels::table.on(arepl::levels::id.eq(arepl::records::level_id)))
            .filter(arepl::levels::legacy.eq(false))
            .filter(arepl::records::is_verification.eq(false))
            .distinct_on(arepl::records::level_id)
            .order_by((
                arepl::records::level_id.asc(),
                arepl::records::achieved_at.asc(),
                arepl::records::created_at.asc(),
                arepl::records::id.asc(),
            ))
            .select((arepl::records::level_id, arepl::records::submitted_by))
            .load::<(Uuid, Uuid)>(conn)?
            .into_iter()
            .filter_map(|(level_id, submitted_by)| (submitted_by == user_id).then_some(level_id))
            .collect::<HashSet<_>>();

        let mut levels_records = HashMap::new();
        for (id, name, position, publisher_id, is_verification) in arepl::records::table
            .inner_join(arepl::levels::table.on(arepl::levels::id.eq(arepl::records::level_id)))
            .filter(arepl::records::submitted_by.eq(user_id))
            .filter(arepl::levels::legacy.eq(false))
            .order(arepl::levels::position.asc())
            .select((
                arepl::levels::id,
                arepl::levels::name,
                arepl::levels::position,
                arepl::levels::publisher_id,
                arepl::records::is_verification,
            ))
            .load::<(Uuid, String, i32, Uuid, bool)>(conn)?
        {
            levels_records
                .entry(id)
                .and_modify(|level: &mut BadgeLevelStatistics| {
                    level.is_verification |= is_verification;
                })
                .or_insert(BadgeLevelStatistics {
                    scope: "platformer",
                    id,
                    name,
                    position,
                    publisher_id,
                    is_verification,
                    is_first_victor: first_victor_level_ids.contains(&id),
                });
        }

        let mut levels_records = levels_records.into_values().collect::<Vec<_>>();
        levels_records.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(left.name.cmp(&right.name))
        });

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
            .order(arepl::levels::position.asc())
            .select((
                arepl::levels::id,
                arepl::levels::name,
                arepl::levels::position,
                arepl::levels::publisher_id,
            ))
            .distinct()
            .load::<(Uuid, String, i32, Uuid)>(conn)?
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
            .order(arepl::levels::position.asc())
            .select((
                arepl::levels::id,
                arepl::levels::name,
                arepl::levels::position,
                arepl::levels::publisher_id,
            ))
            .load::<(Uuid, String, i32, Uuid)>(conn)?
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
        created_levels.sort_by_key(|level| level.position);
        created_levels.dedup_by_key(|level| (level.id, level.publisher_id));

        let completed_level_tags = arepl::records::table
            .inner_join(arepl::levels::table.on(arepl::levels::id.eq(arepl::records::level_id)))
            .filter(arepl::records::submitted_by.eq(user_id))
            .filter(arepl::levels::legacy.eq(false))
            .select((arepl::records::level_id, arepl::levels::tags))
            .distinct()
            .load::<(Uuid, Vec<Option<String>>)>(conn)?;

        let level_tag_counts = Self::count_level_tags(completed_level_tags);

        Ok(Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
        })
    }

    fn count_level_tags(
        completed_level_tags: Vec<(Uuid, Vec<Option<String>>)>,
    ) -> HashMap<String, i64> {
        let mut level_tag_counts = HashMap::new();
        for (_, tags) in completed_level_tags {
            for tag in tags.into_iter().flatten() {
                *level_tag_counts.entry(tag).or_insert(0) += 1;
            }
        }
        level_tag_counts
    }

    fn combine(classic: &Self, platformer: &Self) -> Self {
        let mut levels_records = classic.levels_records.clone();
        levels_records.extend(platformer.levels_records.clone());
        levels_records.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(right.is_verification.cmp(&left.is_verification))
                .then(left.scope.cmp(right.scope))
                .then(left.name.cmp(&right.name))
        });
        levels_records.dedup_by_key(|level| (level.scope, level.id));

        let mut created_levels = classic.created_levels.clone();
        created_levels.extend(platformer.created_levels.clone());
        created_levels.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
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

        Self {
            levels_records,
            created_levels,
            packs,
            level_tag_counts,
        }
    }
}
