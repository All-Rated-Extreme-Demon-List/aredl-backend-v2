use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::user_badges;
use crate::users::badges::badges_list::{
    AvailableBadges, TagBadgeMode, HARDEST_PACK_TIERS, LEVEL_TAG_BADGES, NLW_TIERS,
};
use crate::users::badges::statistics::UserStatistics;
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    delete, insert_into, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, Selectable,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Insertable)]
#[diesel(table_name = user_badges, check_for_backend(Pg))]
pub struct UserBadge {
    #[serde(skip_serializing, skip_deserializing)]
    pub user_id: Uuid,
    /// The code identifying the badge, e.g. "global.level_completion.10".
    pub badge_code: String,
    /// Additional user-specific badge information.
    pub description: Option<String>,
    /// The timestamp when the badge was first unlocked.
    pub unlocked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct UserBadgeGrant {
    /// The code identifying the badge, e.g. "global.level_completion.10".
    pub badge_code: String,
    /// Additional user-specific badge information.
    pub description: Option<String>,
}

impl UserBadge {
    pub fn find_all(conn: &mut DbConnection, user_id: Uuid) -> Result<Vec<Self>, ApiError> {
        Ok(user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .order((
                user_badges::unlocked_at.asc(),
                user_badges::badge_code.asc(),
            ))
            .select(UserBadge::as_select())
            .load::<UserBadge>(conn)?)
    }

    pub fn grant(
        conn: &mut DbConnection,
        user_id: Uuid,
        badge: UserBadgeGrant,
    ) -> Result<Vec<Self>, ApiError> {
        Self::validate_badge_code(&badge.badge_code)?;

        insert_into(user_badges::table)
            .values(UserBadge {
                user_id,
                badge_code: badge.badge_code,
                description: badge.description,
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
    ) -> Result<Vec<Self>, ApiError> {
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
}

impl UserBadge {
    pub fn update_user_badges(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        let badge_data = UserStatistics::load(conn, user_id)?.get_unlocked_badges();
        Self::insert_missing(conn, user_id, badge_data)
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

    fn insert_missing(
        conn: &mut DbConnection,
        user_id: Uuid,
        badges: HashMap<String, Option<String>>,
    ) -> Result<(), ApiError> {
        let existing_codes = user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .select(user_badges::badge_code)
            .load::<String>(conn)?
            .into_iter()
            .collect::<HashSet<_>>();

        let now = Utc::now();
        let new_rows = badges
            .iter()
            .filter(|(badge_code, _)| !existing_codes.contains(*badge_code))
            .map(|(badge_code, description)| UserBadge {
                user_id,
                badge_code: badge_code.clone(),
                description: description.clone(),
                unlocked_at: now,
            })
            .collect::<Vec<_>>();

        if !new_rows.is_empty() {
            insert_into(user_badges::table)
                .values(&new_rows)
                .on_conflict_do_nothing()
                .execute(conn)?;
        }

        Ok(())
    }

    fn validate_badge_code(badge_code: &str) -> Result<(), ApiError> {
        if !AvailableBadges::get_all()
            .iter()
            .any(|badge| badge == badge_code)
        {
            return Err(ApiError::new(
                400,
                &format!("Unknown badge code: {badge_code}"),
            ));
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
    fn get_unlocked_badges(&self) -> HashMap<String, Option<String>> {
        AvailableBadges::get_all()
            .iter()
            .filter(|badge_code| self.is_badge_unlocked(badge_code))
            .map(|badge_code| (badge_code.to_string(), self.badge_description(badge_code)))
            .collect()
    }

    fn is_badge_unlocked(&self, badge_code: &str) -> bool {
        let parts = badge_code.split('.').collect::<Vec<_>>();

        let Some((scope, kind)) = parts.split_first() else {
            return false;
        };

        let scope_statistics = match *scope {
            "classic" => &self.classic,
            "platformer" => &self.platformer,
            "global" => &self.global,
            _ => return false,
        };

        match (*scope, kind) {
            (_, ["level_completion", threshold]) => {
                threshold.parse::<i64>().ok().is_some_and(|threshold| {
                    scope_statistics
                        .levels_records
                        .iter()
                        .map(|level| (level.scope, level.id, level.publisher_id))
                        .collect::<HashSet<_>>()
                        .len() as i64
                        >= threshold
                })
            }
            (_, ["pack_completion", threshold]) => {
                threshold.parse::<i64>().ok().is_some_and(|threshold| {
                    scope_statistics
                        .packs
                        .iter()
                        .map(|pack| (pack.scope, pack.id, pack.name.as_str()))
                        .collect::<HashSet<_>>()
                        .len() as i64
                        >= threshold
                })
            }
            (_, ["level_tags", alias, threshold]) => threshold
                .parse::<i64>()
                .ok()
                .zip(
                    LEVEL_TAG_BADGES
                        .iter()
                        .find(|(level_tag_alias, _, _)| *level_tag_alias == *alias),
                )
                .map(|(threshold, (_, level_tags, mode))| match mode {
                    TagBadgeMode::And => level_tags.iter().all(|tag_name| {
                        scope_statistics
                            .level_tag_counts
                            .get(*tag_name)
                            .is_some_and(|count| *count >= threshold)
                    }),
                    TagBadgeMode::Or => {
                        level_tags
                            .iter()
                            .filter_map(|tag_name| scope_statistics.level_tag_counts.get(*tag_name))
                            .sum::<i64>()
                            >= threshold
                    }
                })
                .unwrap_or(false),
            (_, ["hardest_level", threshold]) => {
                threshold.parse::<i32>().ok().is_some_and(|threshold| {
                    scope_statistics
                        .levels_records
                        .iter()
                        .map(|level| level.position)
                        .min()
                        .is_some_and(|position| position <= threshold)
                })
            }
            (_, ["leaderboard_rank", threshold]) => {
                threshold.parse::<i32>().ok().is_some_and(|threshold| {
                    scope_statistics
                        .leaderboard_rank
                        .is_some_and(|rank| rank <= threshold)
                })
            }
            (_, ["hardest_pack_tier", tier_code]) => scope_statistics
                .packs
                .iter()
                .filter_map(|pack| {
                    HARDEST_PACK_TIERS
                        .iter()
                        .position(|(_, tier_name)| *tier_name == pack.tier_name)
                })
                .max()
                .zip(
                    HARDEST_PACK_TIERS
                        .iter()
                        .position(|(code, _)| *code == *tier_code),
                )
                .is_some_and(|(unlocked_tier_index, required_tier_index)| {
                    unlocked_tier_index >= required_tier_index
                }),
            ("global", ["publisher_levels", threshold]) => {
                threshold.parse::<i64>().ok().is_some_and(|threshold| {
                    let mut publisher_counts = HashMap::new();
                    for level in &scope_statistics.levels_records {
                        *publisher_counts.entry(level.publisher_id).or_insert(0) += 1;
                    }

                    publisher_counts
                        .into_values()
                        .max()
                        .is_some_and(|count| count >= threshold)
                })
            }
            ("global", ["alphabet"]) => {
                let initials = scope_statistics
                    .levels_records
                    .iter()
                    .filter_map(|level| {
                        level
                            .name
                            .chars()
                            .find(|character| character.is_ascii_alphabetic())
                            .map(|character| character.to_ascii_uppercase())
                    })
                    .collect::<HashSet<_>>();

                ('A'..='Z').all(|letter| initials.contains(&letter))
            }
            ("global", ["all_nlw"]) => {
                !NLW_TIERS.is_empty()
                    && NLW_TIERS.iter().all(|required_tier| {
                        scope_statistics.levels_records.iter().any(|level| {
                            level
                                .nlw_tier
                                .as_deref()
                                .is_some_and(|tier| tier == *required_tier)
                        })
                    })
            }
            ("global", ["edel_high"]) => {
                scope_statistics
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
            ("global", ["edel_low"]) => {
                scope_statistics
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
            ("global", ["2p_and_solo"]) => {
                let mut level_versions = HashMap::new();
                for level in &scope_statistics.levels_records {
                    let versions = level_versions
                        .entry((level.scope, level.level_id))
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
            ("global", ["first_victor"]) => scope_statistics
                .levels_records
                .iter()
                .any(|level| level.is_first_victor),
            ("platformer", ["fastest_time"]) => scope_statistics
                .levels_records
                .iter()
                .any(|level| level.is_fastest_time),
            ("global", ["creator"]) => !scope_statistics.created_levels.is_empty(),
            ("global", ["verifier"]) => scope_statistics
                .levels_records
                .iter()
                .any(|level| level.is_verification),
            _ => false,
        }
    }

    fn badge_description(&self, badge_code: &str) -> Option<String> {
        let parts = badge_code.split('.').collect::<Vec<_>>();

        match parts.as_slice() {
            ["classic", "hardest_level", _] => self
                .classic
                .levels_records
                .iter()
                .min_by(|left, right| {
                    left.position
                        .cmp(&right.position)
                        .then(left.name.cmp(&right.name))
                })
                .map(|level| level.name.clone()),
            // description should be the longest list of levels by the same publisher completed by the user
            ["global", "publisher_levels", _] => {
                let mut publisher_levels = HashMap::new();
                for level in &self.global.levels_records {
                    publisher_levels
                        .entry(level.publisher_id)
                        .or_insert_with(Vec::new)
                        .push(level);
                }

                let levels = publisher_levels.into_values().max_by(|left, right| {
                    left.len().cmp(&right.len()).then_with(|| {
                        right
                            .iter()
                            .map(|level| level.position)
                            .min()
                            .cmp(&left.iter().map(|level| level.position).min())
                    })
                })?;

                Self::levels_to_text(
                    levels
                        .into_iter()
                        .map(|level| (level.position, level.name.as_str()))
                        .collect(),
                )
            }
            ["global", "first_victor"] => Self::levels_to_text(
                self.global
                    .levels_records
                    .iter()
                    .filter(|level| level.is_first_victor)
                    .map(|level| (level.position, level.name.as_str()))
                    .collect(),
            ),
            ["platformer", "fastest_time"] => Self::levels_to_text(
                self.platformer
                    .levels_records
                    .iter()
                    .filter(|level| level.is_fastest_time)
                    .map(|level| (level.position, level.name.as_str()))
                    .collect(),
            ),
            ["global", "creator"] => Self::levels_to_text(
                self.global
                    .created_levels
                    .iter()
                    .map(|level| (level.position, level.name.as_str()))
                    .collect(),
            ),
            ["global", "verifier"] => Self::levels_to_text(
                self.global
                    .levels_records
                    .iter()
                    .filter(|level| level.is_verification)
                    .map(|level| (level.position, level.name.as_str()))
                    .collect(),
            ),
            _ => None,
        }
    }

    // creates a text list of levels names from a list of levels
    fn levels_to_text(mut levels: Vec<(i32, &str)>) -> Option<String> {
        if levels.is_empty() {
            return None;
        }

        levels.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(right.1)));

        let level_names = levels.into_iter().map(|(_, name)| name).collect::<Vec<_>>();

        match level_names.as_slice() {
            [] => None,
            [level_name] => Some(level_name.to_string()),
            [first_level, second_level] => Some(format!("{first_level} and {second_level}")),
            _ => {
                let (last_level, other_levels) = level_names.split_last()?;
                Some(format!("{}, and {}", other_levels.join(", "), last_level))
            }
        }
    }
}
