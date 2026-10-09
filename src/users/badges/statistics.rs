use std::collections::{HashMap, HashSet};

use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::prelude::*;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    list::bounty::BountyType,
    list::levels::LevelStatus,
    list::List,
    schema::{
        badge_level_statistics, bounties, bounty_completed, completed_packs, levels,
        levels_created, pack_tiers, packs, user_leaderboard,
    },
};

#[derive(Debug)]
pub struct UserStatistics {
    pub classic: UserListStatistics,
    pub platformer: UserListStatistics,
    pub global: UserListStatistics,
}

#[derive(Debug, Clone, Default)]
pub struct UserListStatistics {
    pub levels_records: Vec<BadgeLevelStatistics>,
    pub created_levels: Vec<BadgeCreatedLevelStatistics>,
    pub packs: Vec<BadgePackStatistics>,
    pub level_tag_counts: HashMap<String, i64>,
    pub bounty_counts: HashMap<String, i64>,
    pub leaderboard_rank: Option<i32>,
    pub level_completion_count: usize,
    pub pack_completion_count: usize,
}

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = badge_level_statistics, check_for_backend(Pg))]
pub struct BadgeLevelStatistics {
    pub list_id: List,
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

#[derive(Debug, Clone, Queryable, Selectable)]
#[diesel(table_name = levels, check_for_backend(Pg))]
pub struct BadgeCreatedLevelStatistics {
    pub list_id: List,
    pub id: Uuid,
    pub name: String,
    pub position: Option<i32>,
}

#[derive(Debug, Clone)]
pub struct BadgePackStatistics {
    pub list_id: List,
    pub id: Uuid,
    pub name: String,
    pub tier_name: String,
}

impl UserStatistics {
    pub fn load(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let mut statistics = Self {
            classic: UserListStatistics::default(),
            platformer: UserListStatistics::default(),
            global: UserListStatistics::default(),
        };

        statistics.classic.leaderboard_rank = user_leaderboard::table
            .filter(user_leaderboard::list_id.eq(List::Classic))
            .filter(user_leaderboard::user_id.eq(user_id))
            .select(user_leaderboard::rank)
            .first::<i32>(conn)
            .optional()?;

        let records = badge_level_statistics::table
            .filter(badge_level_statistics::list_id.eq_any([List::Classic, List::Platformer]))
            .filter(badge_level_statistics::submitted_by.eq(user_id))
            .order((
                badge_level_statistics::position.asc().nulls_last(),
                badge_level_statistics::name.asc(),
            ))
            .select(BadgeLevelStatistics::as_select())
            .load::<BadgeLevelStatistics>(conn)?;
        for record in records {
            statistics
                .list_mut(record.list_id)
                .levels_records
                .push(record);
        }

        let packs = completed_packs::table
            .filter(completed_packs::list_id.eq_any([List::Classic, List::Platformer]))
            .inner_join(packs::table.inner_join(pack_tiers::table))
            .filter(completed_packs::user_id.eq(user_id))
            .order(pack_tiers::placement.asc())
            .select((
                completed_packs::list_id,
                packs::id,
                packs::name,
                pack_tiers::name,
            ))
            .load::<(List, Uuid, String, String)>(conn)?;
        for (list_id, id, name, tier_name) in packs {
            statistics
                .list_mut(list_id)
                .packs
                .push(BadgePackStatistics {
                    list_id,
                    id,
                    name,
                    tier_name,
                });
        }

        let created_levels = levels::table
            .filter(levels::status.ne(LevelStatus::Removed))
            .filter(
                levels::publisher_id.eq(user_id).or(diesel::dsl::exists(
                    levels_created::table
                        .filter(levels_created::level_id.eq(levels::id))
                        .filter(levels_created::user_id.eq(user_id)),
                )),
            )
            .order(levels::position.asc().nulls_last())
            .select(BadgeCreatedLevelStatistics::as_select())
            .load::<BadgeCreatedLevelStatistics>(conn)?;
        for level in created_levels {
            statistics
                .list_mut(level.list_id)
                .created_levels
                .push(level);
        }

        let bounty_counts = bounty_completed::table
            .filter(bounty_completed::list_id.eq_any([List::Classic, List::Platformer]))
            .inner_join(bounties::table)
            .filter(bounty_completed::user_id.eq(user_id))
            .group_by((bounties::list_id, bounties::bounty_type))
            .select((
                bounties::list_id,
                bounties::bounty_type,
                diesel::dsl::count_star(),
            ))
            .load::<(List, BountyType, i64)>(conn)?;
        for key in ["bounty", "weekly", "monthly", "event"] {
            statistics.classic.bounty_counts.insert(key.to_owned(), 0);
            statistics
                .platformer
                .bounty_counts
                .insert(key.to_owned(), 0);
        }
        for (list, bounty_type, count) in bounty_counts {
            let key = match bounty_type {
                BountyType::Bounty => "bounty",
                BountyType::Weekly => "weekly",
                BountyType::Monthly => "monthly",
                BountyType::Event => "event",
            };
            statistics
                .list_mut(list)
                .bounty_counts
                .insert(key.to_owned(), count);
        }

        statistics.classic.count();
        statistics.platformer.count();
        statistics.global =
            UserListStatistics::combine(&statistics.classic, &statistics.platformer);
        Ok(statistics)
    }

    fn list_mut(&mut self, list: List) -> &mut UserListStatistics {
        match list {
            List::Classic => &mut self.classic,
            List::Platformer => &mut self.platformer,
        }
    }
}

impl UserListStatistics {
    fn count(&mut self) {
        self.level_tag_counts = Self::count_level_tags(&self.levels_records);
        self.level_completion_count = self
            .levels_records
            .iter()
            .map(|level| (level.list_id, level.id, level.publisher_id))
            .collect::<HashSet<_>>()
            .len();
        self.pack_completion_count = self
            .packs
            .iter()
            .map(|pack| (pack.list_id, pack.id, pack.name.as_str()))
            .collect::<HashSet<_>>()
            .len();
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
                .then(i16::from(left.list_id).cmp(&i16::from(right.list_id)))
                .then(left.name.cmp(&right.name))
        });
        levels_records.dedup_by_key(|level| (level.list_id, level.id));

        let mut created_levels = classic.created_levels.clone();
        created_levels.extend(platformer.created_levels.clone());
        created_levels.sort_by(|left, right| {
            left.position
                .unwrap_or(i32::MAX)
                .cmp(&right.position.unwrap_or(i32::MAX))
                .then(i16::from(left.list_id).cmp(&i16::from(right.list_id)))
                .then(left.name.cmp(&right.name))
        });
        created_levels.dedup_by_key(|level| (level.list_id, level.id));

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
            level_completion_count: classic.level_completion_count
                + platformer.level_completion_count,
            pack_completion_count: classic.pack_completion_count + platformer.pack_completion_count,
        }
    }
}
