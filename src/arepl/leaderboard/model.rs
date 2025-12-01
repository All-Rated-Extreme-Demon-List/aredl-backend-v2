use crate::app_data::db::DbConnection;
use crate::arepl::levels::BaseLevel;
use crate::clans::Clan;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::scheduled::refresh_matviews::MatviewRefreshLog;
use crate::schema::{
    arepl::{levels, user_leaderboard},
    clans, matview_refresh_log, users,
};
use crate::users::ExtendedBaseUser;
use chrono::Utc;
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=user_leaderboard, check_for_backend(Pg))]
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
    pub user: ExtendedBaseUser,
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

impl LeaderboardPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: LeaderboardQueryOptions,
    ) -> Result<Paginated<Self>, ApiError> {
        let build_filtered_query = || {
            let mut q = user_leaderboard::table
                .inner_join(users::table.on(users::id.eq(user_leaderboard::user_id)))
                .left_join(clans::table.on(user_leaderboard::clan_id.eq(clans::id.nullable())))
                .left_join(levels::table.on(user_leaderboard::hardest.eq(levels::id.nullable())))
                .into_boxed::<Pg>();

            if let Some(name_like) = options.name_filter.clone() {
                q = q.filter(users::global_name.ilike(name_like));
            }

            if let Some(country) = options.country_filter {
                q = q.filter(user_leaderboard::country.eq(country));
            }

            if let Some(clan_id) = options.clan_filter {
                q = q.filter(user_leaderboard::clan_id.eq(clan_id));
            }
            q
        };

        let total_count: i64 = build_filtered_query().count().get_result(conn)?;

        let mut query = build_filtered_query();

        match (
            options.country_filter.is_some(),
            options.order.unwrap_or(LeaderboardOrder::TotalPoints),
        ) {
            (false, LeaderboardOrder::TotalPoints) => {
                query = query.order(user_leaderboard::rank.asc())
            }
            (false, LeaderboardOrder::ExtremeCount) => {
                query = query.order(user_leaderboard::extremes_rank.asc())
            }
            (false, LeaderboardOrder::RawPoints) => {
                query = query.order(user_leaderboard::raw_rank.asc())
            }
            (true, LeaderboardOrder::TotalPoints) => {
                query = query.order(user_leaderboard::country_rank.asc())
            }
            (true, LeaderboardOrder::ExtremeCount) => {
                query = query.order(user_leaderboard::country_extremes_rank.asc())
            }
            (true, LeaderboardOrder::RawPoints) => {
                query = query.order(user_leaderboard::country_raw_rank.asc())
            }
        };

        let query = query.then_order_by(user_leaderboard::user_id.asc());

        let raw: Vec<(
            LeaderboardEntry,
            ExtendedBaseUser,
            Option<Clan>,
            Option<BaseLevel>,
        )> = query
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                LeaderboardEntry::as_select(),
                ExtendedBaseUser::as_select(),
                Option::<Clan>::as_select(),
                Option::<BaseLevel>::as_select(),
            ))
            .load(conn)?;

        let entries_resolved = raw
            .into_iter()
            .map(|(e, user, clan, lvl)| LeaderboardEntryResolved {
                rank: e.rank,
                extremes_rank: e.extremes_rank,
                raw_rank: e.raw_rank,
                country_rank: e.country_rank,
                country_extremes_rank: e.country_extremes_rank,
                country_raw_rank: e.country_raw_rank,
                user,
                country: e.country,
                total_points: e.total_points,
                pack_points: e.pack_points,
                hardest: lvl,
                extremes: e.extremes,
                clan,
            })
            .collect::<Vec<_>>();

        let refresh_log: MatviewRefreshLog = matview_refresh_log::table
            .find("arepl.user_leaderboard")
            .first(conn)
            .unwrap_or(MatviewRefreshLog {
                view_name: "arepl.user_leaderboard".into(),
                last_refresh: Utc::now(),
            });

        Ok(Paginated::<Self>::from_data(
            page_query,
            total_count,
            Self {
                last_refreshed: refresh_log.last_refresh,
                data: entries_resolved,
            },
        ))
    }
}
