use crate::aredl::leaderboard::LeaderboardOrder;
use crate::aredl::levels::BaseLevel;
use crate::clans::Clan;
use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::scheduled::refresh_matviews::MatviewRefreshLog;
use crate::schema::{
    aredl::{clans_leaderboard, levels},
    clans, matview_refresh_log,
};
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
#[diesel(table_name=clans_leaderboard, check_for_backend(Pg))]
pub struct ClansLeaderboardEntry {
    pub rank: i32,
    pub extremes_rank: i32,
    pub clan_id: Uuid,
    pub level_points: i32,
    pub members_count: i32,
    pub hardest: Option<Uuid>,
    pub extremes: i32,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ClansLeaderboardEntryResolved {
    /// Rank of the clan, sorted by total points (including packs).
    pub rank: i32,
    /// Rank of the clan, sorted by count of extremes completed.
    pub extremes_rank: i32,
    /// This entry's clan id.
    pub clan: Clan,
    /// Total points of the country.
    pub level_points: i32,
    /// Count of members in this clan.
    pub members_count: i32,
    /// Hardest level completed by a user in this clan.
    pub hardest: Option<BaseLevel>,
    /// Count of extremes completed by users in this clan.
    pub extremes: i32,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ClansLeaderboardPage {
    /// The last time the leaderboard was refreshed.
    pub last_refreshed: chrono::DateTime<Utc>,
    /// List of leaderboard entries.
    pub data: Vec<ClansLeaderboardEntryResolved>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClansLeaderboardQueryOptions {
    pub order: Option<LeaderboardOrder>,
    pub name_filter: Option<String>,
}

impl ClansLeaderboardPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: ClansLeaderboardQueryOptions,
    ) -> Result<Paginated<Self>, ApiError> {
        let build_filtered_query = || {
            let mut q = clans_leaderboard::table
                .inner_join(clans::table.on(clans::id.eq(clans_leaderboard::clan_id)))
                .left_join(levels::table.on(clans_leaderboard::hardest.eq(levels::id.nullable())))
                .into_boxed::<Pg>();

            if let Some(ref filter) = options.name_filter {
                q = q.filter(clans::global_name.ilike(filter));
            }

            q
        };

        let total_count: i64 = build_filtered_query().count().get_result(conn)?;

        let mut query = build_filtered_query();

        match options.order.unwrap_or(LeaderboardOrder::TotalPoints) {
            LeaderboardOrder::TotalPoints => {
                query = query.order(clans_leaderboard::rank.asc());
            }
            LeaderboardOrder::ExtremeCount => {
                query = query.order(clans_leaderboard::extremes_rank.asc());
            }
            LeaderboardOrder::RawPoints => {
                query = query.order(clans_leaderboard::rank.asc());
            }
        }

        query = query.then_order_by(clans_leaderboard::clan_id.asc());

        let raw_entries: Vec<(ClansLeaderboardEntry, Clan, Option<BaseLevel>)> = query
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                ClansLeaderboardEntry::as_select(),
                Clan::as_select(),
                (levels::id, levels::name).nullable(),
            ))
            .load(conn)?;

        let entries_resolved = raw_entries
            .into_iter()
            .map(|(entry, clan, hardest)| ClansLeaderboardEntryResolved {
                rank: entry.rank,
                extremes_rank: entry.extremes_rank,
                clan,
                level_points: entry.level_points,
                members_count: entry.members_count,
                hardest,
                extremes: entry.extremes,
            })
            .collect::<Vec<_>>();

        let refresh_log: MatviewRefreshLog = matview_refresh_log::table
            .find("aredl.clans_leaderboard")
            .first(conn)
            .unwrap_or(MatviewRefreshLog {
                view_name: "aredl.clans_leaderboard".into(),
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
