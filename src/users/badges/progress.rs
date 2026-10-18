use super::{
    badges_list::{AvailableBadges, BadgeRule, PackTier, TagBadgeMode, NLW_TIERS},
    statistics::UserStatistics,
};
use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    list::{bounty::BountyType, List},
    schema::user_badge_progress,
};
use chrono::{DateTime, Utc};
use diesel::{prelude::*, upsert::excluded};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use strum::IntoEnumIterator as _;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeProgress {
    #[serde(flatten)]
    pub statistics: BadgeProgressStatistics,
    /// When the cached progress metrics last changed.
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BadgeProgressStatistics {
    pub classic: BadgeListProgress,
    pub platformer: BadgeListProgress,
    pub global: BadgeListProgress,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct BadgeListProgress {
    pub level_completion_count: usize,
    pub pack_completion_count: usize,
    pub level_tag_counts: BTreeMap<String, usize>,
    pub bounty_counts: BTreeMap<String, i64>,
    pub leaderboard_rank: Option<i32>,
    pub hardest_pack_tier: Option<PackTier>,
    pub max_same_publisher: usize,
    pub alphabet_letters: String,
    pub nlw_tiers: Vec<String>,
    pub edel_high_count: usize,
    pub edel_low_count: usize,
    pub two_player_and_solo: bool,
    pub first_victor_count: usize,
    pub created_level_count: usize,
    pub verified_level_count: usize,
    pub fastest_time_count: usize,
    pub hardest_level_ranks: BTreeMap<i32, Option<i32>>,
}

impl BadgeListProgress {
    fn from_statistics(
        statistics: &UserStatistics,
        list: Option<List>,
        badges: &[BadgeRule],
    ) -> Self {
        let records = statistics.levels_records(list).collect::<Vec<_>>();
        let created_levels = statistics.created_levels(list).collect::<Vec<_>>();
        let packs = statistics.packs(list).collect::<Vec<_>>();
        let mut level_tag_counts = BTreeMap::new();
        let mut alphabet_letters = BTreeSet::new();
        let mut level_versions = HashMap::new();
        let mut edel_high_count = 0;
        let mut edel_low_count = 0;
        for record in &records {
            for tag in record.tags.iter().flatten() {
                *level_tag_counts.entry(tag.clone()).or_insert(0) += 1;
            }
            if let Some(initial) = record.name.chars().find(char::is_ascii_alphabetic) {
                alphabet_letters.insert(initial.to_ascii_uppercase());
            }
            let versions = level_versions
                .entry((record.list_id, record.level_id))
                .or_insert((false, false));
            if record.two_player {
                versions.1 = true;
            } else {
                versions.0 = true;
            }
            if record
                .edel_enjoyment
                .is_some_and(|enjoyment| enjoyment >= 90.0)
            {
                edel_high_count += 1;
            }
            if record
                .edel_enjoyment
                .is_some_and(|enjoyment| enjoyment <= 40.0)
            {
                edel_low_count += 1;
            }
        }
        let nlw_tiers = NLW_TIERS
            .iter()
            .filter(|tier| {
                records
                    .iter()
                    .any(|level| level.nlw_tier.as_deref() == Some(**tier))
            })
            .map(|tier| (*tier).to_owned())
            .collect();
        let mut bounty_counts = [
            BountyType::Bounty,
            BountyType::Weekly,
            BountyType::Monthly,
            BountyType::Event,
        ]
        .into_iter()
        .map(|kind| (kind.as_ref().to_owned(), 0))
        .collect::<BTreeMap<_, _>>();
        for (bounty_list, kind, count) in &statistics.bounty_counts {
            if list.is_none_or(|list| *bounty_list == list) {
                *bounty_counts.entry(kind.as_ref().to_owned()).or_insert(0) += *count;
            }
        }
        let mut hardest_level_ranks = BTreeMap::new();
        for &badge in badges {
            let BadgeRule::HardestLevel {
                list: badge_list,
                threshold,
                cutoff,
            } = badge
            else {
                continue;
            };
            if Some(badge_list) != list {
                continue;
            }
            let rank = records
                .iter()
                .filter_map(|level| UserStatistics::hardest_level_rank(level, cutoff))
                .min();
            hardest_level_ranks.insert(threshold, rank);
        }
        Self {
            level_completion_count: records.len(),
            pack_completion_count: packs
                .iter()
                .map(|pack| (pack.list_id, pack.id))
                .collect::<HashSet<_>>()
                .len(),
            level_tag_counts,
            bounty_counts,
            leaderboard_rank: match list {
                Some(List::Classic) => statistics.leaderboard_rank,
                Some(List::Platformer) | None => None,
            },
            hardest_pack_tier: packs
                .iter()
                .filter_map(|pack| PackTier::iter().find(|tier| tier.name() == pack.tier_name))
                .max(),
            max_same_publisher: statistics.max_publisher_levels(list).len(),
            alphabet_letters: alphabet_letters.into_iter().collect(),
            nlw_tiers,
            edel_high_count,
            edel_low_count,
            two_player_and_solo: level_versions
                .into_values()
                .any(|(has_solo, has_two_player)| has_solo && has_two_player),
            first_victor_count: records.iter().filter(|level| level.is_first_victor).count(),
            created_level_count: created_levels.len(),
            verified_level_count: records.iter().filter(|level| level.is_verification).count(),
            fastest_time_count: records.iter().filter(|level| level.is_fastest_time).count(),
            hardest_level_ranks,
        }
    }
}

impl From<&UserStatistics> for BadgeProgressStatistics {
    fn from(statistics: &UserStatistics) -> Self {
        let badges = AvailableBadges::get_all();
        Self {
            classic: BadgeListProgress::from_statistics(statistics, Some(List::Classic), &badges),
            platformer: BadgeListProgress::from_statistics(
                statistics,
                Some(List::Platformer),
                &badges,
            ),
            global: BadgeListProgress::from_statistics(statistics, None, &badges),
        }
    }
}

impl BadgeProgressStatistics {
    pub(super) fn is_badge_unlocked(&self, badge: BadgeRule) -> bool {
        let statistics = match badge.list() {
            Some(List::Classic) => &self.classic,
            Some(List::Platformer) => &self.platformer,
            None => &self.global,
        };
        match badge {
            BadgeRule::LevelCompletion(threshold) => statistics.level_completion_count >= threshold,
            BadgeRule::PackCompletion(threshold) => statistics.pack_completion_count >= threshold,
            BadgeRule::BountyBoard {
                bounty_type,
                threshold,
            } => statistics
                .bounty_counts
                .get(bounty_type.as_ref())
                .is_some_and(|count| *count >= threshold),
            BadgeRule::LevelTags {
                tags,
                mode,
                threshold,
                ..
            } => match mode {
                TagBadgeMode::And => tags.iter().all(|tag| {
                    statistics
                        .level_tag_counts
                        .get(*tag)
                        .is_some_and(|count| *count >= threshold)
                }),
                TagBadgeMode::Or => {
                    tags.iter()
                        .filter_map(|tag| statistics.level_tag_counts.get(*tag))
                        .sum::<usize>()
                        >= threshold
                }
            },
            BadgeRule::HardestLevel { threshold, .. } => statistics
                .hardest_level_ranks
                .get(&threshold)
                .is_some_and(|rank| rank.is_some_and(|rank| rank <= threshold)),
            BadgeRule::LeaderboardRank(threshold) => statistics
                .leaderboard_rank
                .is_some_and(|rank| rank <= threshold),
            BadgeRule::HardestPackTier(tier) => statistics
                .hardest_pack_tier
                .is_some_and(|unlocked| unlocked >= tier),
            BadgeRule::PublisherLevels(threshold) => statistics.max_same_publisher >= threshold,
            BadgeRule::Alphabet => statistics.alphabet_letters.len() == 26,
            BadgeRule::AllNlw => {
                !NLW_TIERS.is_empty()
                    && NLW_TIERS.iter().all(|tier| {
                        statistics
                            .nlw_tiers
                            .iter()
                            .any(|completed| completed == tier)
                    })
            }
            BadgeRule::EdelHigh => statistics.edel_high_count >= 3,
            BadgeRule::EdelLow => statistics.edel_low_count >= 3,
            BadgeRule::TwoPlayerAndSolo => statistics.two_player_and_solo,
            BadgeRule::FirstVictor => statistics.first_victor_count > 0,
            BadgeRule::FastestTime => statistics.fastest_time_count > 0,
            BadgeRule::Creator => statistics.created_level_count > 0,
            BadgeRule::Verifier => statistics.verified_level_count > 0,
        }
    }
}

impl UserBadgeProgress {
    pub fn find(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        let cached = user_badge_progress::table
            .find(user_id)
            .select((
                user_badge_progress::statistics,
                user_badge_progress::updated_at,
            ))
            .first::<(serde_json::Value, DateTime<Utc>)>(conn)
            .optional()?;

        let (statistics, updated_at) = if let Some(cached) = cached {
            cached
        } else {
            let statistics = BadgeProgressStatistics::from(&UserStatistics::load(conn, user_id)?);
            let value = serde_json::to_value(statistics).map_err(|error| {
                ApiError::InternalServerError(format!(
                    "Failed to serialize badge progress: {error}"
                ))
            })?;
            diesel::insert_into(user_badge_progress::table)
                .values((
                    user_badge_progress::user_id.eq(user_id),
                    user_badge_progress::statistics.eq(value),
                ))
                .on_conflict_do_nothing()
                .returning((
                    user_badge_progress::statistics,
                    user_badge_progress::updated_at,
                ))
                .get_result(conn)?
        };
        Ok(Self {
            statistics: serde_json::from_value(statistics).map_err(|error| {
                ApiError::InternalServerError(format!("Failed to read badge progress: {error}"))
            })?,
            updated_at,
        })
    }

    pub(super) fn save(
        conn: &mut DbConnection,
        user_id: Uuid,
        statistics: BadgeProgressStatistics,
    ) -> Result<Self, ApiError> {
        let value = serde_json::to_value(&statistics).map_err(|error| {
            ApiError::InternalServerError(format!("Failed to serialize badge progress: {error}"))
        })?;
        let cached = user_badge_progress::table
            .find(user_id)
            .select((
                user_badge_progress::statistics,
                user_badge_progress::updated_at,
            ))
            .first::<(serde_json::Value, DateTime<Utc>)>(conn)
            .optional()?;
        if let Some((previous, updated_at)) = cached {
            if previous == value {
                return Ok(Self {
                    statistics,
                    updated_at,
                });
            }
        }

        let updated_at = Utc::now();
        diesel::insert_into(user_badge_progress::table)
            .values((
                user_badge_progress::user_id.eq(user_id),
                user_badge_progress::statistics.eq(value),
                user_badge_progress::updated_at.eq(updated_at),
            ))
            .on_conflict(user_badge_progress::user_id)
            .do_update()
            .set((
                user_badge_progress::statistics.eq(excluded(user_badge_progress::statistics)),
                user_badge_progress::updated_at.eq(excluded(user_badge_progress::updated_at)),
            ))
            .execute(conn)?;
        Ok(Self {
            statistics,
            updated_at,
        })
    }
}
