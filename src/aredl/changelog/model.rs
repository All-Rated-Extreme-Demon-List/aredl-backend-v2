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
pub struct ChangelogEntry {
    pub new_position: Option<i32>,
    pub old_position: Option<i32>,
    pub legacy: Option<bool>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_levels, check_for_backend(Pg))]
pub struct Level {
    pub id: Uuid,
    pub name: String,
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

impl ChangelogEntryResolved {
    pub fn find_all<const D: i64>(page_query: PageQuery<D>) -> Result<Paginated<Self>, ApiError> {
        let (level_affected, level_above, level_below) = diesel::alias!(
            aredl_levels as level_affected,
            aredl_levels as level_above,
            aredl_levels as level_below,
        );

        let records: Vec<(ChangelogEntry, Level, Option<Level>, Option<Level>)> =
            aredl_position_history::table
                .order(aredl_position_history::i.desc())
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .inner_join(level_affected.on(aredl_position_history::affected_level.eq(level_affected.field(aredl_levels::id))))
                .left_join(level_above.on(aredl_position_history::level_above.eq(level_above.field(aredl_levels::id).nullable())))
                .left_join(level_below.on(aredl_position_history::level_below.eq(level_below.field(aredl_levels::id).nullable())))
                .select((
                    ChangelogEntry::as_select(),
                    level_affected.fields((aredl_levels::id, aredl_levels::name)),
                    level_above.fields((aredl_levels::id, aredl_levels::name)).nullable(),
                    level_below.fields((aredl_levels::id, aredl_levels::name)).nullable(),
                ))
                .load::<(ChangelogEntry, Level, Option<Level>, Option<Level>)>(&mut db::connection()?)?;

        let records_resolved = records.into_iter().map(|(entry, affected, above, below)|
            ChangelogEntryResolved {
                new_position: entry.new_position,
                old_position: entry.old_position,
                legacy: entry.legacy,
                created_at: entry.created_at,
                affected_level: affected,
                level_above: above,
                level_below: below,
            }
        ).collect::<Vec<_>>();

        let count: i64 = aredl_position_history::table.count().get_result(&mut db::connection()?)?;
        let pages = (count / page_query.per_page()) + 1;

        Ok(Paginated::<Self>::from_data(page_query, pages, records_resolved))
    }
}