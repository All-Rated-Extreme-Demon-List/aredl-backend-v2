use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::user_badges;
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

const BADGES: &[&str] = &[
    "classic.hardest_level.800",
    "classic.hardest_level.400",
    "classic.hardest_level.250",
    "classic.hardest_level.150",
    "classic.hardest_level.75",
    "platformer.level_completion.1",
    "global.level_completion.1",
    "global.level_completion.5",
    "global.level_completion.10",
    "global.level_completion.25",
    "global.level_completion.50",
    "global.level_completion.100",
    "global.pack_completion.1",
    "global.pack_completion.5",
    "global.pack_completion.10",
    "global.hardest_pack_tier.iron",
    "global.hardest_pack_tier.gold",
    "global.hardest_pack_tier.ruby",
    "global.hardest_pack_tier.sapphire",
    "global.hardest_pack_tier.pearl",
    "global.hardest_pack_tier.diamond",
    "global.publisher_levels.3",
    "global.publisher_levels.5",
    "global.level_tags.xxlplus",
    "global.level_tags.bossfight",
    "global.level_tags.alltags",
    "global.alphabet",
    "global.first_victor",
    "global.creator",
    "global.verifier",
];

// (badge code, [(level tag, required count)])
const LEVEL_TAG_BADGES: &[(&str, &[(&str, i64)])] = &[
    (
        "alltags",
        &[
            ("2P", 1),
            ("Circles", 1),
            ("Clicksync", 1),
            ("Fast-Paced", 1),
            ("Timings", 1),
            ("Chokepoints", 1),
            ("Learny", 1),
            ("Memory", 1),
            ("High CPS", 1),
            ("Gimmicky", 1),
            ("Flow", 1),
            ("Slow-Paced", 1),
            ("Precision", 1),
            ("Bossfight", 1),
            ("Mirror", 1),
            ("Nerve Control", 1),
            ("Cube", 1),
            ("Ship", 1),
            ("Ball", 1),
            ("UFO", 1),
            ("Wave", 1),
            ("Robot", 1),
            ("Spider", 1),
            ("Old Swing", 1),
            ("New Swing", 1),
            ("Duals", 1),
            ("Overall", 1),
        ],
    ),
    ("xxlplus", &[("XXL+", 5)]),
    ("bossfight", &[("Bossfight", 5)]),
];

// hardcoded here to not bother with dynamic fetching, they never change
const HARDEST_PACK_TIERS: &[(&str, &str)] = &[
    ("iron", "Iron Tier"),
    ("gold", "Gold Tier"),
    ("ruby", "Ruby Tier"),
    ("sapphire", "Sapphire Tier"),
    ("pearl", "Pearl Tier"),
    ("diamond", "Diamond Tier"),
];

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, Queryable, Selectable, Insertable)]
#[diesel(table_name = user_badges, check_for_backend(Pg))]
pub struct UserBadge {
    #[serde(skip_serializing, skip_deserializing)]
    pub user_id: Uuid,
    /// The code identifying the badge, e.g. "global.level_completion.10".
    pub badge_code: String,
    /// The timestamp when the badge was first unlocked.
    pub unlocked_at: DateTime<Utc>,
}

impl UserBadge {
    pub fn update_user_badges(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        let badge_codes = UserStatistics::load(conn, user_id)?.get_unlocked_badges();
        Self::insert_missing(conn, user_id, badge_codes)
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

    pub fn grant_all(
        conn: &mut DbConnection,
        user_id: Uuid,
        badge_codes: Vec<String>,
    ) -> Result<Vec<Self>, ApiError> {
        Self::validate_badge_codes(&badge_codes)?;

        let now = Utc::now();
        let new_rows = badge_codes
            .into_iter()
            .map(|badge_code| UserBadge {
                user_id,
                badge_code,
                unlocked_at: now,
            })
            .collect::<Vec<_>>();

        if !new_rows.is_empty() {
            insert_into(user_badges::table)
                .values(&new_rows)
                .on_conflict_do_nothing()
                .execute(conn)?;
        }

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

    fn insert_missing(
        conn: &mut DbConnection,
        user_id: Uuid,
        badge_codes: HashSet<String>,
    ) -> Result<(), ApiError> {
        let existing_codes = user_badges::table
            .filter(user_badges::user_id.eq(user_id))
            .select(user_badges::badge_code)
            .load::<String>(conn)?
            .into_iter()
            .collect::<HashSet<_>>();

        let now = Utc::now();
        let new_rows = badge_codes
            .into_iter()
            .filter(|code| !existing_codes.contains(code))
            .map(|badge_code| UserBadge {
                user_id,
                badge_code,
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

    fn validate_badge_codes(badge_codes: &[String]) -> Result<(), ApiError> {
        if let Some(invalid_badge_code) = badge_codes
            .iter()
            .find(|badge_code| !BADGES.contains(&badge_code.as_str()))
        {
            return Err(ApiError::new(
                400,
                &format!("Unknown badge code: {invalid_badge_code}"),
            ));
        }

        Ok(())
    }
}

impl UserStatistics {
    fn get_unlocked_badges(&self) -> HashSet<String> {
        BADGES
            .iter()
            .map(|badge_code| badge_code.to_string())
            .filter(|badge_code| self.is_badge_unlocked(badge_code))
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
            (_, ["level_tags", preset_code]) => LEVEL_TAG_BADGES
                .iter()
                .find(|(code, _)| *code == *preset_code)
                .map(|(_, required_tags)| {
                    required_tags.iter().all(|(tag_name, required_count)| {
                        scope_statistics
                            .level_tag_counts
                            .get(*tag_name)
                            .is_some_and(|count| *count >= *required_count)
                    })
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
            ("global", ["first_victor"]) => scope_statistics
                .levels_records
                .iter()
                .any(|level| level.is_first_victor),
            ("global", ["creator"]) => !scope_statistics.created_levels.is_empty(),
            ("global", ["verifier"]) => scope_statistics
                .levels_records
                .iter()
                .any(|level| level.is_verification),
            _ => false,
        }
    }
}
