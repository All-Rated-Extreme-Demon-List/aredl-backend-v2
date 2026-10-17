use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::{levels, records, user_badges};
use crate::users::badges::badges_list::{
    AvailableBadges, BadgeRule, PackTier, TagBadgeMode, NLW_TIERS,
};
use crate::users::badges::statistics::{BadgeLevelStatistics, UserListStatistics, UserStatistics};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::upsert::excluded;
use diesel::{delete, insert_into, Selectable};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use strum::IntoEnumIterator as _;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Debug, Clone, Queryable, Selectable, Insertable)]
#[diesel(table_name = user_badges, check_for_backend(Pg))]
pub struct UserBadge {
    pub user_id: Uuid,
    pub badge_code: String,
    pub level_ids: Vec<Uuid>,
    pub record_id: Option<Uuid>,
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeResolved {
    /// The code identifying the badge, e.g. `global.level_completion.10`.
    pub badge_code: String,
    /// Names of levels associated with this badge if applicable (e.g. verified, created, etc)
    pub levels: Vec<String>,
    /// The completion associated with this badge if applicable (e.g. hardest level)
    pub record: Option<UserBadgeRecord>,
    /// The timestamp when the badge was first unlocked.
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Name of the level associated with this record.
    pub level: String,
    /// Timestamp of when the record was achieved.
    pub achieved_at: DateTime<Utc>,
}

impl From<(Uuid, String, DateTime<Utc>)> for UserBadgeRecord {
    fn from((id, level, achieved_at): (Uuid, String, DateTime<Utc>)) -> Self {
        Self {
            id,
            level,
            achieved_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeGrant {
    /// The code identifying the badge, e.g. `global.level_completion.10`.
    pub badge_code: String,
    /// Internal UUIDs of the levels associated with this badge, if applicable.
    #[serde(default)]
    pub level_ids: Vec<Uuid>,
    /// Internal UUID of the user's completion associated with this badge, if applicable.
    pub record_id: Option<Uuid>,
}

impl UserBadge {
    pub fn find_all(
        conn: &mut DbConnection,
        user_id: Uuid,
    ) -> Result<Vec<UserBadgeResolved>, ApiError> {
        let badges = user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .order((
                user_badges::unlocked_at.asc(),
                user_badges::badge_code.asc(),
            ))
            .select(Self::as_select())
            .load::<Self>(conn)?;

        let level_ids = badges
            .iter()
            .flat_map(|badge| badge.level_ids.iter().copied())
            .collect::<HashSet<_>>();
        let levels_by_id = if level_ids.is_empty() {
            HashMap::new()
        } else {
            levels::table
                .filter(levels::id.eq_any(level_ids))
                .select((levels::id, levels::name))
                .load::<(Uuid, String)>(conn)?
                .into_iter()
                .collect::<HashMap<_, _>>()
        };

        let record_ids = badges
            .iter()
            .filter_map(|badge| badge.record_id)
            .collect::<HashSet<_>>();
        let records_by_id = if record_ids.is_empty() {
            HashMap::new()
        } else {
            records::table
                .inner_join(levels::table)
                .filter(records::id.eq_any(record_ids))
                .filter(records::submitted_by.eq(user_id))
                .select((records::id, levels::name, records::achieved_at))
                .load::<(Uuid, String, DateTime<Utc>)>(conn)?
                .into_iter()
                .map(UserBadgeRecord::from)
                .map(|record| (record.id, record))
                .collect::<HashMap<_, _>>()
        };

        Ok(badges
            .into_iter()
            .map(|badge| UserBadgeResolved {
                badge_code: badge.badge_code,
                levels: badge
                    .level_ids
                    .iter()
                    .filter_map(|id| levels_by_id.get(id).cloned())
                    .collect(),
                record: badge
                    .record_id
                    .and_then(|id| records_by_id.get(&id).cloned()),
                unlocked_at: badge.unlocked_at,
            })
            .collect())
    }

    pub fn grant(
        conn: &mut DbConnection,
        user_id: Uuid,
        mut badge: UserBadgeGrant,
    ) -> Result<Vec<UserBadgeResolved>, ApiError> {
        Self::validate_badge_code(&badge.badge_code)?;

        let mut level_ids = HashSet::new();
        badge.level_ids.retain(|id| level_ids.insert(*id));
        if !badge.level_ids.is_empty() {
            let existing = levels::table
                .filter(levels::id.eq_any(&badge.level_ids))
                .select(levels::id)
                .load::<Uuid>(conn)?
                .into_iter()
                .collect::<HashSet<_>>();
            if let Some(id) = badge.level_ids.iter().find(|id| !existing.contains(id)) {
                return Err(ApiError::BadRequest(format!("Unknown level ID: {id}")));
            }
        }
        if let Some(record_id) = badge.record_id {
            let record = records::table
                .filter(records::id.eq(record_id))
                .filter(records::submitted_by.eq(user_id))
                .select(records::id)
                .first::<Uuid>(conn)
                .optional()?;
            if record.is_none() {
                return Err(ApiError::BadRequest(format!(
                    "Record {record_id} does not belong to this user"
                )));
            }
        }

        insert_into(user_badges::table)
            .values(Self {
                user_id,
                badge_code: badge.badge_code,
                level_ids: badge.level_ids,
                record_id: badge.record_id,
                unlocked_at: Utc::now(),
            })
            .on_conflict_do_nothing()
            .execute(conn)?;

        Self::find_all(conn, user_id)
    }

    pub fn remove_all(
        conn: &mut DbConnection,
        user_id: Uuid,
        badge_codes: Vec<String>,
    ) -> Result<Vec<UserBadgeResolved>, ApiError> {
        Self::validate_badge_codes(&badge_codes)?;

        if !badge_codes.is_empty() {
            delete(
                user_badges::table
                    .filter(user_badges::user_id.eq(user_id))
                    .filter(user_badges::badge_code.eq_any(badge_codes)),
            )
            .execute(conn)?;
        }

        Self::find_all(conn, user_id)
    }

    pub fn update_user_badges(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        let statistics = UserStatistics::load(conn, user_id)?;
        let already_unlocked_badges = user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .select((
                user_badges::badge_code,
                user_badges::level_ids,
                user_badges::record_id,
            ))
            .load::<(String, Vec<Uuid>, Option<Uuid>)>(conn)?
            .into_iter()
            .map(|(code, level_ids, record_id)| (code, (level_ids, record_id)))
            .collect::<HashMap<_, _>>();

        let now = Utc::now();
        // includes all new and already unlocked ones
        let new_unlocked_badges = AvailableBadges::get_all()
            .into_iter()
            .filter_map(|badge| {
                let badge_code = badge.to_string();
                let current_badge_data = already_unlocked_badges.get(&badge_code);
                if current_badge_data.is_none() && !statistics.is_badge_unlocked(badge) {
                    return None;
                }
                let (level_ids, record_id) = statistics.badge_additional_data(badge);
                if current_badge_data.is_some_and(|(existing_level_ids, existing_record_id)| {
                    existing_level_ids == &level_ids && *existing_record_id == record_id
                }) {
                    return None;
                }
                Some(Self {
                    user_id,
                    badge_code,
                    level_ids,
                    record_id,
                    unlocked_at: now,
                })
            })
            .collect::<Vec<_>>();

        if !new_unlocked_badges.is_empty() {
            insert_into(user_badges::table)
                .values(&new_unlocked_badges)
                .on_conflict((user_badges::user_id, user_badges::badge_code))
                .do_update()
                .set((
                    user_badges::level_ids.eq(excluded(user_badges::level_ids)),
                    user_badges::record_id.eq(excluded(user_badges::record_id)),
                ))
                .execute(conn)?;
        }
        Ok(())
    }

    pub fn has_code(
        conn: &mut DbConnection,
        user_id: Uuid,
        badge_code: &str,
    ) -> Result<bool, ApiError> {
        let exists = user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .filter(user_badges::badge_code.eq(badge_code))
            .select(user_badges::user_id)
            .first::<Uuid>(conn)
            .optional()?;

        Ok(exists.is_some())
    }

    fn validate_badge_code(badge_code: &str) -> Result<(), ApiError> {
        if !AvailableBadges::get_all()
            .iter()
            .any(|badge| badge.to_string() == badge_code)
        {
            return Err(ApiError::BadRequest(format!(
                "Unknown badge code: {badge_code}"
            )));
        }

        Ok(())
    }

    fn validate_badge_codes(badge_codes: &[String]) -> Result<(), ApiError> {
        for badge_code in badge_codes {
            Self::validate_badge_code(badge_code)?;
        }

        Ok(())
    }
}
impl UserStatistics {
    fn list_statistics(&self, list: Option<List>) -> &UserListStatistics {
        match list {
            Some(List::Classic) => &self.classic,
            Some(List::Platformer) => &self.platformer,
            None => &self.global,
        }
    }

    fn is_badge_unlocked(&self, badge: BadgeRule) -> bool {
        let statistics = self.list_statistics(badge.list());

        match badge {
            BadgeRule::LevelCompletion(threshold) => statistics.level_completion_count >= threshold,
            BadgeRule::PackCompletion(threshold) => statistics.pack_completion_count >= threshold,
            BadgeRule::BountyBoard {
                bounty_type,
                threshold,
            } => statistics
                .bounty_counts
                .get(&bounty_type)
                .is_some_and(|count| *count >= threshold),
            BadgeRule::LevelTags {
                tags,
                mode,
                threshold,
                ..
            } => match mode {
                TagBadgeMode::And => tags.iter().all(|tag_name| {
                    statistics
                        .level_tag_counts
                        .get(*tag_name)
                        .is_some_and(|count| *count >= threshold)
                }),
                TagBadgeMode::Or => {
                    tags.iter()
                        .filter_map(|tag_name| statistics.level_tag_counts.get(*tag_name))
                        .sum::<usize>()
                        >= threshold
                }
            },
            BadgeRule::HardestLevel {
                threshold, cutoff, ..
            } => statistics
                .levels_records
                .iter()
                .any(|level| Self::qualifies_for_hardest_level(level, threshold, cutoff)),
            BadgeRule::LeaderboardRank(threshold) => statistics
                .leaderboard_rank
                .is_some_and(|rank| rank <= threshold),
            BadgeRule::HardestPackTier(required_tier) => statistics
                .packs
                .iter()
                .filter_map(|pack| PackTier::iter().find(|tier| tier.name() == pack.tier_name))
                .max()
                .is_some_and(|unlocked_tier| unlocked_tier >= required_tier),
            BadgeRule::PublisherLevels(threshold) => {
                statistics.per_publisher_levels().len() >= threshold
            }
            BadgeRule::Alphabet => {
                let initials = statistics
                    .levels_records
                    .iter()
                    .filter_map(|level| {
                        level
                            .name
                            .chars()
                            .find(char::is_ascii_alphabetic)
                            .map(|character| character.to_ascii_uppercase())
                    })
                    .collect::<HashSet<_>>();

                ('A'..='Z').all(|letter| initials.contains(&letter))
            }
            BadgeRule::AllNlw => {
                !NLW_TIERS.is_empty()
                    && NLW_TIERS.iter().all(|required_tier| {
                        statistics.levels_records.iter().any(|level| {
                            level
                                .nlw_tier
                                .as_deref()
                                .is_some_and(|tier| tier == *required_tier)
                        })
                    })
            }
            BadgeRule::EdelHigh => {
                statistics
                    .levels_records
                    .iter()
                    .filter(|level| {
                        level
                            .edel_enjoyment
                            .is_some_and(|edel_enjoyment| edel_enjoyment >= 90.0)
                    })
                    .count()
                    >= 3
            }
            BadgeRule::EdelLow => {
                statistics
                    .levels_records
                    .iter()
                    .filter(|level| {
                        level
                            .edel_enjoyment
                            .is_some_and(|edel_enjoyment| edel_enjoyment <= 40.0)
                    })
                    .count()
                    >= 3
            }
            BadgeRule::TwoPlayerAndSolo => {
                let mut level_versions = HashMap::new();
                for level in &statistics.levels_records {
                    let versions = level_versions
                        .entry((level.list_id, level.level_id))
                        .or_insert((false, false));

                    if level.two_player {
                        versions.1 = true;
                    } else {
                        versions.0 = true;
                    }
                }

                level_versions
                    .into_values()
                    .any(|(has_solo, has_two_player)| has_solo && has_two_player)
            }
            BadgeRule::FirstVictor => statistics
                .levels_records
                .iter()
                .any(|level| level.is_first_victor),
            BadgeRule::FastestTime => statistics
                .levels_records
                .iter()
                .any(|level| level.is_fastest_time),
            BadgeRule::Creator => !statistics.created_levels.is_empty(),
            BadgeRule::Verifier => statistics
                .levels_records
                .iter()
                .any(|level| level.is_verification),
        }
    }

    fn badge_additional_data(&self, badge: BadgeRule) -> (Vec<Uuid>, Option<Uuid>) {
        let statistics = self.list_statistics(badge.list());
        let levels = match badge {
            BadgeRule::HardestLevel {
                threshold, cutoff, ..
            } => {
                let record = statistics
                    .levels_records
                    .iter()
                    .filter(|level| Self::qualifies_for_hardest_level(level, threshold, cutoff))
                    .min_by(|left, right| {
                        left.current_position
                            .or(left.position)
                            .unwrap_or(i32::MAX)
                            .cmp(
                                &right
                                    .current_position
                                    .or(right.position)
                                    .unwrap_or(i32::MAX),
                            )
                            .then(left.name.cmp(&right.name))
                            .then(left.record_id.cmp(&right.record_id))
                    });
                return (Vec::new(), record.map(|record| record.record_id));
            }
            // description should be the longest list of levels by the same publisher completed by the user
            BadgeRule::PublisherLevels(_) => statistics
                .per_publisher_levels()
                .into_iter()
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect::<Vec<_>>(),
            BadgeRule::FirstVictor => statistics
                .levels_records
                .iter()
                .filter(|level| level.is_first_victor)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::FastestTime => statistics
                .levels_records
                .iter()
                .filter(|level| level.is_fastest_time)
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::Creator => statistics
                .created_levels
                .iter()
                .map(|level| (level.position, level.name.as_str(), level.id))
                .collect(),
            BadgeRule::Verifier => statistics
                .levels_records
                .iter()
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

    // for the hardest level badge, checks if a level completion can award it
    fn qualifies_for_hardest_level(
        level: &BadgeLevelStatistics,
        threshold: i32,
        cutoff: Option<DateTime<Utc>>,
    ) -> bool {
        level
            .current_position
            .is_some_and(|position| position <= threshold)
            || (level.position.is_some_and(|position| position <= threshold)
                && cutoff.is_none_or(|cutoff| level.achieved_at >= cutoff))
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
