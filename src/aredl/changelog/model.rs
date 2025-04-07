use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use diesel::pg::Pg;
use utoipa::ToSchema;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::aredl::levels::BaseLevel;
use crate::schema::{aredl_levels, aredl_position_history};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_position_history, check_for_backend(Pg))]
pub struct ChangelogEntryData {
    /// New position of the level after the action.
    pub new_position: Option<i32>,
    /// Old position of the level before the action.
    pub old_position: Option<i32>,
    /// Whether the level is now in legacy after the action or not.
    pub legacy: Option<bool>,
    /// Timestamp for when the action was performed.
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ChangelogEntry {
    pub action: ChangelogAction,
    /// Timestamp for when the action was performed.
    pub created_at: NaiveDateTime,
    /// The level that was affected by the action.
    pub affected_level: BaseLevel,
    /// The level that is now above the affected level after the action.
    pub level_above: Option<BaseLevel>,
    /// The level that is now below the affected level after the action.
    pub level_below: Option<BaseLevel>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum ChangelogAction {
    /// A new level was placed on the list.
    Placed { 
        /// Position the level was placed at.
        new_position: i32,
        /// Whether the level was placed in the legacy list or not.
        legacy: bool 
    },
    /// An existing level was raised from one position to another.
    Raised { 
        /// New position of the level after the action.
        new_position: i32, 
        /// Previous position of the level before the action.
        old_position: i32 
    },
    /// An existing level was lowered from one position to another.
    Lowered { 
        /// New position of the level after the action.
        new_position: i32, 
        /// Previous position of the level before the action.
        old_position: i32 
    },
    /// An existing level was removed from the list.
    Removed { 
        /// Position of the level before it was removed.
        old_position: i32 
    },
    /// An existing level was swapped with another level.
    Swapped { 
        /// Position of the upper level after the action.
        upper_position: i32, 
        /// The upper level out of the two that were swapped.
        upper_level: BaseLevel, 
        /// The lower level out of the two that were swapped.
        other_level: BaseLevel
    },
    /// An existing level was moved to the legacy list after being rerated to insane.
    MovedToLegacy { 
        /// New position of the level after the action.
        new_position: i32, 
        /// Previous position of the level before the action.
        old_position: i32 
    },
    /// An existing level was moved from the legacy list after being rerated to extreme.
    MovedFromLegacy { 
        /// New position of the level after the action.
        new_position: i32, 
        /// Previous position of the level before the action.
        old_position: i32 
    },
    /// An unknown action was performed.
    Unknown { 
        /// New position of the level after the action.
        new_position: Option<i32>, 
        /// Previous position of the level before the action.
        old_position: Option<i32>, 
        /// Whether the level is in the legacy list after the action or not.
        legacy: Option<bool>
    },
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ChangelogPage {
    /// List of changelog entries.
    pub data: Vec<ChangelogEntry>
}

impl ChangelogPage {
    pub fn find<const D: i64>(db: web::Data<Arc<DbAppState>>, page_query: PageQuery<D>) -> Result<Paginated<Self>, ApiError> {
        let (level_affected, level_above, level_below) = diesel::alias!(
            aredl_levels as level_affected,
            aredl_levels as level_above,
            aredl_levels as level_below,
        );

        let records: Vec<(ChangelogEntryData, BaseLevel, Option<BaseLevel>, Option<BaseLevel>)> =
            aredl_position_history::table
                .order(aredl_position_history::i.desc())
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .inner_join(level_affected.on(aredl_position_history::affected_level.eq(level_affected.field(aredl_levels::id))))
                .left_join(level_above.on(aredl_position_history::level_above.eq(level_above.field(aredl_levels::id).nullable())))
                .left_join(level_below.on(aredl_position_history::level_below.eq(level_below.field(aredl_levels::id).nullable())))
                .select((
                    ChangelogEntryData::as_select(),
                    level_affected.fields((aredl_levels::id, aredl_levels::name)),
                    level_above.fields((aredl_levels::id, aredl_levels::name)).nullable(),
                    level_below.fields((aredl_levels::id, aredl_levels::name)).nullable(),
                ))
                .load::<(ChangelogEntryData, BaseLevel, Option<BaseLevel>, Option<BaseLevel>)>(&mut db.connection()?)?;

        let records_resolved = records.into_iter().map(|(entry, affected, above, below)| {
            let action = ChangelogAction::from_data(&entry, &affected, &above, &below);
            ChangelogEntry {
                created_at: entry.created_at,
                affected_level: affected,
                level_above: above,
                level_below: below,
                action,
            }
        }).collect::<Vec<_>>();

        let count: i64 = aredl_position_history::table.count().get_result(&mut db.connection()?)?;

        Ok(Paginated::<Self>::from_data(page_query, count, Self {
            data: records_resolved
        }))
    }
}

impl ChangelogAction {
    pub fn from_data(entry: &ChangelogEntryData, level: &BaseLevel, level_above: &Option<BaseLevel>, level_below: &Option<BaseLevel>) -> Self {
        match (entry.legacy, entry.new_position, entry.old_position) {
            (Some(legacy), Some(new_position), None) => Self::Placed { new_position, legacy },
            (None, Some(new_position), Some(old_position)) => {
                let unknown = Self::Unknown {
                    new_position: Some(new_position),
                    old_position: Some(old_position),
                    legacy: None,
                };
                match (new_position < old_position, (new_position - old_position).abs(), level_above, level_below) {
                    (_, 0, _, _) => unknown,
                    (true, 1, Some(other_level), _) => Self::Swapped { upper_position: new_position, upper_level: level.clone(), other_level: other_level.clone() },
                    (false, 1, _, Some(other_level)) => Self::Swapped { upper_position: old_position, upper_level: other_level.clone(), other_level: other_level.clone() },
                    (_, 1, _, _) => unknown,
                    (true, _, _, _) => Self::Raised { new_position, old_position },
                    (false, _, _, _) => Self::Lowered { new_position, old_position }
                }
            }
            (None, None, Some(old_position)) => Self::Removed { old_position },
            (Some(true), Some(new_position), Some(old_position)) => Self::MovedToLegacy { new_position, old_position },
            (Some(false), Some(new_position), Some(old_position)) => Self::MovedFromLegacy { new_position, old_position },
            (legacy, new_position, old_position) => Self::Unknown { new_position, old_position, legacy },
        }
    }
}