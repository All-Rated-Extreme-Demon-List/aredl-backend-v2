use chrono::NaiveDateTime;
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use crate::db;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{aredl_levels, aredl_position_history};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_position_history, check_for_backend(Pg))]
pub struct ChangelogEntryData {
    pub new_position: Option<i32>,
    pub old_position: Option<i32>,
    pub legacy: Option<bool>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Clone, Debug)]
#[diesel(table_name=aredl_levels, check_for_backend(Pg))]
pub struct Level {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangelogEntry {
    pub action: ChangelogAction,
    pub created_at: NaiveDateTime,
    pub affected_level: Level,
    pub level_above: Option<Level>,
    pub level_below: Option<Level>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ChangelogAction {
    Placed { new_position: i32, legacy: bool },
    Raised { new_position: i32, old_position: i32 },
    Lowered { new_position: i32, old_position: i32 },
    Removed { old_position: i32 },
    Swapped { upper_position: i32, upper_level: Level, other_level: Level},
    MovedToLegacy { new_position: i32, old_position: i32 },
    MovedFromLegacy { new_position: i32, old_position: i32 },
    Unknown { new_position: Option<i32>, old_position: Option<i32>, legacy: Option<bool>},
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChangelogEntryResolved {
    pub new_position: Option<i32>,
    pub old_position: Option<i32>,
    pub legacy: Option<bool>,
    pub created_at: NaiveDateTime,
    pub affected_level: Level,
    pub level_above: Option<Level>,
    pub level_below: Option<Level>,
}

impl ChangelogEntry {
    pub fn find_all<const D: i64>(page_query: PageQuery<D>) -> Result<Paginated<Self>, ApiError> {
        let (level_affected, level_above, level_below) = diesel::alias!(
            aredl_levels as level_affected,
            aredl_levels as level_above,
            aredl_levels as level_below,
        );

        let records: Vec<(ChangelogEntryData, Level, Option<Level>, Option<Level>)> =
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
                .load::<(ChangelogEntryData, Level, Option<Level>, Option<Level>)>(&mut db::connection()?)?;

        let records_resolved = records.into_iter().map(|(entry, affected, above, below)| {
            let action = ChangelogAction::from_data(&entry, &affected, &above, &below);
            ChangelogEntry {
                affected_level: affected,
                created_at: entry.created_at,
                level_above: above,
                level_below: below,
                action,
            }
        }
        ).collect::<Vec<_>>();

        let count: i64 = aredl_position_history::table.count().get_result(&mut db::connection()?)?;
        let pages = (count / page_query.per_page()) + 1;

        Ok(Paginated::<Self>::from_data(page_query, pages, records_resolved))
    }
}

impl ChangelogAction {
    pub fn from_data(entry: &ChangelogEntryData, level: &Level, level_above: &Option<Level>, level_below: &Option<Level>) -> Self {
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
                    (true, 1, _, Some(other_level)) => Self::Swapped { upper_position: new_position, upper_level: level.clone(), other_level: other_level.clone() },
                    (false, 1, Some(other_level), _) => Self::Swapped { upper_position: old_position, upper_level: other_level.clone(), other_level: other_level.clone() },
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