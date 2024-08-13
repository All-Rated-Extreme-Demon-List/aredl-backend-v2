use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{BoolExpressionMethods, BoxableExpression, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::expression::AsExpression;
use diesel::expression::expression_types::NotSelectable;
use diesel::sql_types::{Bool, Nullable};
use serde::{Deserialize, Serialize};
use crate::custom_schema::aredl_user_leaderboard;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{aredl_levels, users};

#[derive(Serialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_user_leaderboard, check_for_backend(Pg))]
pub struct LeaderboardEntry {
    pub rank: i32,
    pub extremes_rank: i32,
    pub raw_rank: i32,
    pub country_rank: i32,
    pub country_extremes_rank: i32,
    pub country_raw_rank: i32,
    pub user_id: Uuid,
    pub country: Option<i32>,
    pub total_points: i32,
    pub pack_points: i32,
    pub hardest: Option<Uuid>,
    pub extremes: i32
}

#[derive(Serialize, Selectable, Queryable, Debug)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub global_name: String,
    pub country: Option<i32>,
    pub discord_id: Option<String>,
    pub discord_avatar: Option<String>
}

#[derive(Serialize, Selectable, Queryable, Debug)]
#[diesel(table_name=aredl_levels, check_for_backend(Pg))]
pub struct Level {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Debug)]
pub struct LeaderboardEntryResolved {
    pub rank: i32,
    pub extremes_rank: i32,
    pub raw_rank: i32,
    pub country_rank: i32,
    pub country_extremes_rank: i32,
    pub country_raw_rank: i32,
    pub user: User,
    pub country: Option<i32>,
    pub total_points: i32,
    pub pack_points: i32,
    pub hardest: Option<Level>,
    pub extremes: i32
}

#[derive(Serialize, Debug)]
pub struct LeaderboardPage {
    pub data: Vec<LeaderboardEntryResolved>
}

#[derive(Serialize, Deserialize, Debug)]
pub enum LeaderboardOrder {
    TotalPoints,
    RawPoints,
    ExtremeCount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardQueryOptions {
    pub name_filter: Option<String>,
    pub country_filter: Option<i32>,
    pub order: Option<LeaderboardOrder>,
}

impl LeaderboardPage {
    pub fn find<const D: i64>(conn: &mut DbConnection, page_query: PageQuery<D>, options: LeaderboardQueryOptions) -> Result<Paginated<Self>, ApiError> {
        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> = match options.name_filter.clone() {
            Some(filter) => Box::new(users::global_name.ilike(filter)),
            None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
        };
        let country_filter: Box<dyn BoxableExpression<_, _, SqlType = Nullable<Bool>>> = match options.country_filter.clone() {
            Some(country) => Box::new(aredl_user_leaderboard::country.is_not_null()
                .and(aredl_user_leaderboard::country.eq(country))),
            None => Box::new(<bool as AsExpression<Bool>>::as_expression(true).nullable()),
        };

        let selection = (
            LeaderboardEntry::as_select(),
            User::as_select(),
            (aredl_levels::id, aredl_levels::name).nullable()
        );

        let order = options.order.unwrap_or(LeaderboardOrder::TotalPoints);

        let ordering: Box< dyn BoxableExpression<_, _, SqlType = NotSelectable>> =
            match (options.country_filter, order) {
                (None, LeaderboardOrder::TotalPoints) => Box::new(aredl_user_leaderboard::rank.asc()),
                (None, LeaderboardOrder::ExtremeCount) => Box::new(aredl_user_leaderboard::extremes_rank.asc()),
                (None, LeaderboardOrder::RawPoints) => Box::new(aredl_user_leaderboard::raw_rank.asc()),
                (Some(_), LeaderboardOrder::TotalPoints) => Box::new(aredl_user_leaderboard::country_rank.asc()),
                (Some(_), LeaderboardOrder::ExtremeCount) => Box::new(aredl_user_leaderboard::country_extremes_rank.asc()),
                (Some(_), LeaderboardOrder::RawPoints) => Box::new(aredl_user_leaderboard::country_raw_rank.asc()),
            };

        let query =
            aredl_user_leaderboard::table
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .inner_join(users::table.on(users::id.eq(aredl_user_leaderboard::user_id)))
                .left_join(aredl_levels::table.on(aredl_user_leaderboard::hardest.eq(aredl_levels::id.nullable())));

        let entries = query.clone()
            .filter(name_filter)
            .filter(country_filter)
            .order((ordering, aredl_user_leaderboard::user_id))
            .select(selection)
            .load::<(LeaderboardEntry, User, Option<Level>)>(conn)?;

        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> = match options.name_filter {
            Some(filter) => Box::new(users::global_name.ilike(filter)),
            None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
        };
        let country_filter: Box<dyn BoxableExpression<_, _, SqlType = Nullable<Bool>>> = match options.country_filter {
            Some(country) => Box::new(aredl_user_leaderboard::country.eq(country)),
            None => Box::new(<bool as AsExpression<Bool>>::as_expression(true).nullable()),
        };

        let count = query
            .filter(name_filter)
            .filter(country_filter)
            .count()
            .get_result(conn)?;

        let entries_resolved = entries
            .into_iter()
            .map(|(entry, user, hardest)| LeaderboardEntryResolved {
                rank: entry.rank,
                extremes_rank: entry.extremes_rank,
                raw_rank: entry.raw_rank,
                country_rank: entry.country_rank,
                country_extremes_rank: entry.country_extremes_rank,
                country_raw_rank: entry.country_raw_rank,
                user,
                country: entry.country,
                total_points: entry.total_points,
                pack_points: entry.pack_points,
                hardest,
                extremes: entry.extremes
            })
            .collect::<Vec<_>>();

        Ok(Paginated::<Self>::from_data(page_query, count, Self {
            data: entries_resolved
        }))
    }
}