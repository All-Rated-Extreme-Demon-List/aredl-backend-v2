use super::badges_list::BadgeRule;
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, prelude::*};
use itertools::Itertools as _;
use std::{cmp::Reverse, collections::HashSet};
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    list::{bounty::BountyType, levels::LevelStatus, List},
    schema::{
        badge_level_statistics, bounties, bounty_completed, completed_packs, levels,
        levels_created, pack_tiers, packs, user_leaderboard,
    },
};

#[derive(Debug)]
pub struct UserStatistics {
    pub levels_records: Vec<BadgeLevelStatistics>,
    pub created_levels: Vec<BadgeCreatedLevelStatistics>,
    pub packs: Vec<BadgePackStatistics>,
    pub bounty_counts: Vec<(List, BountyType, i64)>,
    pub leaderboard_rank: Option<i32>,
}

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = badge_level_statistics, check_for_backend(Pg))]
pub struct BadgeLevelStatistics {
    pub list_id: List,
    pub id: Uuid,
    pub record_id: Uuid,
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

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = levels, check_for_backend(Pg))]
pub struct BadgeCreatedLevelStatistics {
    pub list_id: List,
    pub id: Uuid,
    pub name: String,
    pub position: Option<i32>,
}

#[derive(Debug, Queryable)]
pub struct BadgePackStatistics {
    pub list_id: List,
    pub id: Uuid,
    pub tier_name: String,
}

impl UserStatistics {
    pub fn load(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let leaderboard_rank = user_leaderboard::table
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

        let packs = completed_packs::table
            .filter(completed_packs::list_id.eq_any([List::Classic, List::Platformer]))
            .inner_join(packs::table.inner_join(pack_tiers::table))
            .filter(completed_packs::user_id.eq(user_id))
            .order(pack_tiers::placement.asc())
            .select((completed_packs::list_id, packs::id, pack_tiers::name))
            .load::<BadgePackStatistics>(conn)?;

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

        Ok(Self {
            levels_records: records,
            created_levels,
            packs,
            bounty_counts,
            leaderboard_rank,
        })
    }
}

impl UserStatistics {
    pub fn levels_records(
        &self,
        list: Option<List>,
    ) -> impl Iterator<Item = &BadgeLevelStatistics> + Clone {
        self.levels_records
            .iter()
            .filter(move |record| list.is_none_or(|list| record.list_id == list))
    }

    pub fn created_levels(
        &self,
        list: Option<List>,
    ) -> impl Iterator<Item = &BadgeCreatedLevelStatistics> {
        self.created_levels
            .iter()
            .filter(move |level| list.is_none_or(|list| level.list_id == list))
    }

    pub fn packs(&self, list: Option<List>) -> impl Iterator<Item = &BadgePackStatistics> {
        self.packs
            .iter()
            .filter(move |pack| list.is_none_or(|list| pack.list_id == list))
    }

    pub fn max_publisher_levels(&self, list: Option<List>) -> Vec<&BadgeLevelStatistics> {
        self.levels_records(list)
            .into_group_map_by(|level| level.publisher_id)
            .into_iter()
            .max_by_key(|(publisher_id, levels)| {
                (
                    levels.len(),
                    Reverse(levels.iter().filter_map(|level| level.position).min()),
                    Reverse(*publisher_id),
                )
            })
            .map(|(_, levels)| levels)
            .unwrap_or_default()
    }

    pub fn hardest_level_rank(
        level: &BadgeLevelStatistics,
        cutoff: Option<DateTime<Utc>>,
    ) -> Option<i32> {
        level
            .current_position
            .into_iter()
            .chain(
                level
                    .position
                    .filter(|_| cutoff.is_none_or(|cutoff| level.achieved_at >= cutoff)),
            )
            .min()
    }

    pub fn badge_additional_data(&self, badge: BadgeRule) -> (Vec<Uuid>, Option<Uuid>) {
        let list = badge.list();
        let levels = match badge {
            BadgeRule::HardestLevel {
                threshold, cutoff, ..
            } => {
                let record = self
                    .levels_records(list)
                    .filter(|level| {
                        Self::hardest_level_rank(level, cutoff)
                            .is_some_and(|rank| rank <= threshold)
                    })
                    .min_by_key(|level| {
                        (
                            level
                                .current_position
                                .or(level.position)
                                .unwrap_or(i32::MAX),
                            level.name.as_str(),
                            level.record_id,
                        )
                    });
                return (Vec::new(), record.map(|record| record.record_id));
            }
            // should be the longest list of levels by the same publisher completed by the user
            BadgeRule::PublisherLevels(_) => self
                .max_publisher_levels(list)
                .into_iter()
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect::<Vec<_>>(),
            BadgeRule::FirstVictor => self
                .levels_records(list)
                .filter(|level| level.is_first_victor)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::FastestTime => self
                .levels_records(list)
                .filter(|level| level.is_fastest_time)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::Creator => self
                .created_levels(list)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::Verifier => self
                .levels_records(list)
                .filter(|level| level.is_verification)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::LevelCompletion(_)
            | BadgeRule::LeaderboardRank(_)
            | BadgeRule::PackCompletion(_)
            | BadgeRule::BountyBoard { .. }
            | BadgeRule::HardestPackTier(_)
            | BadgeRule::LevelTags { .. }
            | BadgeRule::AllNlw
            | BadgeRule::EdelHigh
            | BadgeRule::EdelLow
            | BadgeRule::TwoPlayerAndSolo
            | BadgeRule::Alphabet => Vec::new(),
        };
        (Self::level_ids(levels), None)
    }

    fn level_ids<'a>(levels: impl IntoIterator<Item = (Option<i32>, &'a str, Uuid)>) -> Vec<Uuid> {
        let mut levels = levels.into_iter().collect::<Vec<_>>();
        levels.sort_by(|left, right| {
            left.0
                .unwrap_or(i32::MAX)
                .cmp(&right.0.unwrap_or(i32::MAX))
                .then(left.1.cmp(right.1))
                .then(left.2.cmp(&right.2))
        });
        let mut added_levels = HashSet::new();
        levels
            .into_iter()
            .map(|(_, _, id)| id)
            .filter(|id| added_levels.insert(*id))
            .collect()
    }
}
