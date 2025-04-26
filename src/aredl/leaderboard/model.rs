use crate::aredl::levels::BaseLevel;
use crate::clans::Clan;
use crate::custom_schema::aredl_user_leaderboard;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{aredl_levels, clans, matview_refresh_log, users};
use crate::users::BaseDiscordUser;
use chrono::Utc;
use diesel::expression::expression_types::NotSelectable;
use diesel::expression::AsExpression;
use diesel::pg::Pg;
use diesel::sql_types::{Bool, Nullable};
use diesel::{
    BoolExpressionMethods, BoxableExpression, ExpressionMethods, JoinOnDsl,
    NullableExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
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
    pub extremes: i32,
    pub clan_id: Option<Uuid>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct LeaderboardEntryResolved {
    /// Rank of the user in the global leaderboard, sorted by total points (including packs).
    pub rank: i32,
    /// Rank of the user in the global leaderboard, sorted by count of extremes completed.
    pub extremes_rank: i32,
    /// Rank of the user in the global leaderboard, sorted by total points (excluding packs).
    pub raw_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by total points (including packs).
    pub country_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by count of extremes completed.
    pub country_extremes_rank: i32,
    /// Rank of the user in the country leaderboard, sorted by total points (excluding packs).
    pub country_raw_rank: i32,
    /// This entry's user.
    pub user: BaseDiscordUser,
    /// Country of the user. Uses the ISO 3166-1 numeric country code.
    pub country: Option<i32>,
    /// Total points of the user, including pack points.
    pub total_points: i32,
    /// Pack points of the user.
    pub pack_points: i32,
    /// Hardest level the user has completed.
    pub hardest: Option<BaseLevel>,
    /// Count of extremes the user has completed.
    pub extremes: i32,
    /// User's clan, if any.
    pub clan: Option<Clan>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct LeaderboardPage {
    /// The last time the leaderboard was refreshed.
    pub last_refreshed: chrono::DateTime<Utc>,
    /// List of leaderboard entries.
    pub data: Vec<LeaderboardEntryResolved>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub enum LeaderboardOrder {
    /// Sort by total points (including packs).
    TotalPoints,
    /// Sort by total points (excluding packs).
    RawPoints,
    /// Sort by count of extremes completed.
    ExtremeCount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LeaderboardQueryOptions {
    pub name_filter: Option<String>,
    pub country_filter: Option<i32>,
    pub clan_filter: Option<Uuid>,
    pub order: Option<LeaderboardOrder>,
}

#[derive(Queryable, Insertable, Debug)]
#[diesel(table_name = matview_refresh_log, check_for_backend(Pg))]
pub struct MatviewRefreshLog {
    pub view_name: String,
    pub last_refresh: chrono::DateTime<Utc>,
}

impl LeaderboardPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: LeaderboardQueryOptions,
    ) -> Result<Paginated<Self>, ApiError> {
        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.name_filter.clone() {
                Some(filter) => Box::new(users::global_name.ilike(filter)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
            };
        let country_filter: Box<dyn BoxableExpression<_, _, SqlType = Nullable<Bool>>> =
            match options.country_filter.clone() {
                Some(country) => Box::new(
                    aredl_user_leaderboard::country
                        .is_not_null()
                        .and(aredl_user_leaderboard::country.eq(country)),
                ),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true).nullable()),
            };

        let clan_filter: Box<dyn BoxableExpression<_, _, SqlType = Nullable<Bool>>> =
            match options.clan_filter.clone() {
                Some(clan_id) => Box::new(
                    aredl_user_leaderboard::clan_id
                        .is_not_null()
                        .and(aredl_user_leaderboard::clan_id.eq(clan_id)),
                ),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true).nullable()),
            };

        let selection = (
            LeaderboardEntry::as_select(),
            BaseDiscordUser::as_select(),
            Option::<Clan>::as_select(),
            Option::<BaseLevel>::as_select(),
        );

        let order = options.order.unwrap_or(LeaderboardOrder::TotalPoints);

        let ordering: Box<dyn BoxableExpression<_, _, SqlType = NotSelectable>> =
            match (options.country_filter, order) {
                (None, LeaderboardOrder::TotalPoints) => {
                    Box::new(aredl_user_leaderboard::rank.asc())
                }
                (None, LeaderboardOrder::ExtremeCount) => {
                    Box::new(aredl_user_leaderboard::extremes_rank.asc())
                }
                (None, LeaderboardOrder::RawPoints) => {
                    Box::new(aredl_user_leaderboard::raw_rank.asc())
                }
                (Some(_), LeaderboardOrder::TotalPoints) => {
                    Box::new(aredl_user_leaderboard::country_rank.asc())
                }
                (Some(_), LeaderboardOrder::ExtremeCount) => {
                    Box::new(aredl_user_leaderboard::country_extremes_rank.asc())
                }
                (Some(_), LeaderboardOrder::RawPoints) => {
                    Box::new(aredl_user_leaderboard::country_raw_rank.asc())
                }
            };

        let query = aredl_user_leaderboard::table
            .inner_join(users::table.on(users::id.eq(aredl_user_leaderboard::user_id)))
            .left_join(clans::table.on(aredl_user_leaderboard::clan_id.eq(clans::id.nullable())))
            .left_join(
                aredl_levels::table
                    .on(aredl_user_leaderboard::hardest.eq(aredl_levels::id.nullable())),
            );

        let entries = query
            .clone()
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .filter(name_filter)
            .filter(country_filter)
            .filter(clan_filter)
            .order((ordering, aredl_user_leaderboard::user_id))
            .select(selection)
            .load::<(
                LeaderboardEntry,
                BaseDiscordUser,
                Option<Clan>,
                Option<BaseLevel>,
            )>(conn)?;

        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.name_filter {
                Some(filter) => Box::new(users::global_name.ilike(filter)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
            };
        let country_filter: Box<dyn BoxableExpression<_, _, SqlType = Nullable<Bool>>> =
            match options.country_filter {
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
            .map(|(entry, user, clan, hardest)| LeaderboardEntryResolved {
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
                extremes: entry.extremes,
                clan,
            })
            .collect::<Vec<_>>();

        let refresh_log: MatviewRefreshLog = matview_refresh_log::table
            .find("aredl_user_leaderboard")
            .first(conn)
            .unwrap_or(MatviewRefreshLog {
                view_name: "aredl_user_leaderboard".into(),
                last_refresh: Utc::now(),
            });

        Ok(Paginated::<Self>::from_data(
            page_query,
            count,
            Self {
                last_refreshed: refresh_log.last_refresh,
                data: entries_resolved,
            },
        ))
    }
}
