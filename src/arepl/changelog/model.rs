use crate::app_data::db::DbConnection;
use crate::arepl::levels::{BaseLevel, LevelStatus};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::arepl::{levels, position_history};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods as _, JoinOnDsl as _, NullableExpressionMethods as _, QueryDsl as _,
    RunQueryDsl as _, Selectable, SelectableHelper as _,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=position_history, check_for_backend(Pg))]
pub struct ChangelogEntryData {
    /// New position of the level after the action.
    pub new_position: Option<i32>,
    /// Old position of the level before the action.
    pub old_position: Option<i32>,
    /// Old status of the level before the action. Can be null for the initial entry of a level.
    pub old_status: Option<LevelStatus>,
    /// New status of the level after the action.
    pub new_status: LevelStatus,
    /// Timestamp for when the action was performed.
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ChangelogEntry {
    pub action: ChangelogAction,
    /// Timestamp for when the action was performed.
    pub created_at: DateTime<Utc>,
    /// The level that was affected by the action.
    pub affected_level: BaseLevel,
    /// The level that is now above the affected level after the action.
    pub level_above: Option<BaseLevel>,
    /// The level that is now below the affected level after the action.
    pub level_below: Option<BaseLevel>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum ChangelogAction {
    /// A new level was added to the list but does not have an actual placement yet.
    Pending,
    /// A new level was placed on the list.
    Placed {
        /// Position the level was placed at.
        new_position: i32,
        /// The status the level was placed into. (Main list or legacy list)
        status: LevelStatus,
    },
    /// An existing level was raised from one position to another.
    Raised {
        /// New position of the level after the action.
        new_position: i32,
        /// Previous position of the level before the action.
        old_position: i32,
    },
    /// An existing level was lowered from one position to another.
    Lowered {
        /// New position of the level after the action.
        new_position: i32,
        /// Previous position of the level before the action.
        old_position: i32,
    },
    /// An existing level was removed from the list.
    Removed {
        /// Position of the level before it was removed.
        old_position: Option<i32>,
    },
    /// An existing level was swapped with another level.
    Swapped {
        /// Position of the upper level after the action.
        upper_position: i32,
        /// The upper level out of the two that were swapped.
        upper_level: BaseLevel,
        /// The lower level out of the two that were swapped.
        other_level: BaseLevel,
    },
    /// An existing level was moved to the legacy list after being rerated to insane.
    MovedToLegacy {
        /// New position of the level after the action.
        new_position: i32,
        /// Previous position of the level before the action.
        old_position: i32,
    },
    /// An existing level was moved from the legacy list after being rerated to extreme.
    MovedFromLegacy {
        /// New position of the level after the action.
        new_position: i32,
        /// Previous position of the level before the action.
        old_position: i32,
    },
    /// An unknown action was performed.
    Unknown {
        /// New position of the level after the action.
        new_position: Option<i32>,
        /// Previous position of the level before the action.
        old_position: Option<i32>,
        /// Old status of the level before the action. Can be null for the initial entry of a level.
        old_status: Option<LevelStatus>,
        /// New status of the level after the action.
        new_status: LevelStatus,
    },
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ChangelogPage {
    /// List of changelog entries.
    pub data: Vec<ChangelogEntry>,
}

impl ChangelogPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
    ) -> Result<Paginated<Self>, ApiError> {
        let (level_affected, level_above, level_below) = diesel::alias!(
            levels as level_affected,
            levels as level_above,
            levels as level_below,
        );

        let records: Vec<(
            ChangelogEntryData,
            BaseLevel,
            Option<BaseLevel>,
            Option<BaseLevel>,
        )> = position_history::table
            .order(position_history::i.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .inner_join(
                level_affected
                    .on(position_history::affected_level.eq(level_affected.field(levels::id))),
            )
            .left_join(
                level_above
                    .on(position_history::level_above.eq(level_above.field(levels::id).nullable())),
            )
            .left_join(
                level_below
                    .on(position_history::level_below.eq(level_below.field(levels::id).nullable())),
            )
            .select((
                ChangelogEntryData::as_select(),
                level_affected.fields(<BaseLevel as Selectable<Pg>>::construct_selection()),
                level_above
                    .fields(<BaseLevel as Selectable<Pg>>::construct_selection())
                    .nullable(),
                level_below
                    .fields(<BaseLevel as Selectable<Pg>>::construct_selection())
                    .nullable(),
            ))
            .load::<(
                ChangelogEntryData,
                BaseLevel,
                Option<BaseLevel>,
                Option<BaseLevel>,
            )>(conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(entry, affected, above, below)| {
                let action =
                    ChangelogAction::from_data(&entry, &affected, above.as_ref(), below.as_ref());
                ChangelogEntry {
                    created_at: entry.created_at,
                    affected_level: affected,
                    level_above: above,
                    level_below: below,
                    action,
                }
            })
            .collect::<Vec<_>>();

        let count: i64 = position_history::table.count().get_result(conn)?;

        Ok(Paginated::<Self>::from_data(
            page_query,
            count,
            Self {
                data: records_resolved,
            },
        ))
    }
}

impl ChangelogAction {
    pub fn from_data(
        entry: &ChangelogEntryData,
        level: &BaseLevel,
        level_above: Option<&BaseLevel>,
        level_below: Option<&BaseLevel>,
    ) -> Self {
        match (
            &entry.old_status,
            &entry.new_status,
            entry.new_position,
            entry.old_position,
        ) {
            (_, LevelStatus::Pending, None, _) => Self::Pending,
            (_, LevelStatus::Removed, _, old_position) => Self::Removed { old_position },
            (_, LevelStatus::MainList | LevelStatus::Legacy, Some(new_position), None) => {
                Self::Placed {
                    new_position,
                    status: entry.new_status.clone(),
                }
            }
            (Some(old_status), new_status, Some(new_position), Some(old_position))
                if old_status == new_status =>
            {
                let unknown = Self::Unknown {
                    new_position: Some(new_position),
                    old_position: Some(old_position),
                    old_status: Some(old_status.clone()),
                    new_status: new_status.clone(),
                };
                match (
                    new_position < old_position,
                    (new_position - old_position).abs(),
                    level_above,
                    level_below,
                ) {
                    (true, 1, _, Some(other_level)) => Self::Swapped {
                        upper_position: new_position,
                        upper_level: level.clone(),
                        other_level: other_level.clone(),
                    },
                    (false, 1, Some(other_level), _) => Self::Swapped {
                        upper_position: old_position,
                        upper_level: other_level.clone(),
                        other_level: other_level.clone(),
                    },
                    (_, 1 | 0, _, _) => unknown,
                    (true, _, _, _) => Self::Raised {
                        new_position,
                        old_position,
                    },
                    (false, _, _, _) => Self::Lowered {
                        new_position,
                        old_position,
                    },
                }
            }
            (
                Some(LevelStatus::MainList),
                LevelStatus::Legacy,
                Some(new_position),
                Some(old_position),
            ) => Self::MovedToLegacy {
                new_position,
                old_position,
            },
            (
                Some(LevelStatus::Legacy),
                LevelStatus::MainList,
                Some(new_position),
                Some(old_position),
            ) => Self::MovedFromLegacy {
                new_position,
                old_position,
            },
            (old_status, new_status, new_position, old_position) => Self::Unknown {
                new_position,
                old_position,
                old_status: old_status.clone(),
                new_status: new_status.clone(),
            },
        }
    }
}
