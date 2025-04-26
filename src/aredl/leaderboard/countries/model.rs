use crate::aredl::leaderboard::{LeaderboardOrder, MatviewRefreshLog};
use crate::aredl::levels::BaseLevel;
use crate::custom_schema::aredl_country_leaderboard;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{aredl_levels, matview_refresh_log};
use chrono::Utc;
use diesel::expression::expression_types::NotSelectable;
use diesel::pg::Pg;
use diesel::{
    BoxableExpression, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl,
    RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_country_leaderboard, check_for_backend(Pg))]
pub struct CountryLeaderboardEntry {
    pub rank: i32,
    pub extremes_rank: i32,
    pub country: i32,
    pub level_points: i32,
    pub members_count: i32,
    pub hardest: Option<Uuid>,
    pub extremes: i32,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct CountryLeaderboardEntryResolved {
    /// Rank of the country, sorted by total points (including packs).
    pub rank: i32,
    /// Rank of the country, sorted by count of extremes completed.
    pub extremes_rank: i32,
    /// This entry's country. Uses the ISO 3166-1 numeric country code.
    pub country: i32,
    /// Total points of the country.
    pub level_points: i32,
    /// Count of members in this country.
    pub members_count: i32,
    /// Hardest level completed by a user in this country.
    pub hardest: Option<BaseLevel>,
    /// Count of extremes completed by users in this country.
    pub extremes: i32,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct CountryLeaderboardPage {
    /// Timestamp of the last refresh of this leaderboard.
    pub last_refresh: chrono::DateTime<Utc>,
    /// List of leaderboard entries.
    pub data: Vec<CountryLeaderboardEntryResolved>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CountryLeaderboardQueryOptions {
    pub order: Option<LeaderboardOrder>,
}

impl CountryLeaderboardPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: CountryLeaderboardQueryOptions,
    ) -> Result<Paginated<Self>, ApiError> {
        let selection = (
            CountryLeaderboardEntry::as_select(),
            (aredl_levels::id, aredl_levels::name).nullable(),
        );

        let order = options.order.unwrap_or(LeaderboardOrder::TotalPoints);

        let ordering: Box<dyn BoxableExpression<_, _, SqlType = NotSelectable>> = match order {
            LeaderboardOrder::TotalPoints => Box::new(aredl_country_leaderboard::rank.asc()),
            LeaderboardOrder::ExtremeCount => {
                Box::new(aredl_country_leaderboard::extremes_rank.asc())
            }
            LeaderboardOrder::RawPoints => Box::new(aredl_country_leaderboard::rank.asc()),
        };

        let query = aredl_country_leaderboard::table.left_join(
            aredl_levels::table
                .on(aredl_country_leaderboard::hardest.eq(aredl_levels::id.nullable())),
        );

        let entries = query
            .clone()
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .order((ordering, aredl_country_leaderboard::country.asc()))
            .select(selection)
            .load::<(CountryLeaderboardEntry, Option<BaseLevel>)>(conn)?;

        let count = query.count().get_result(conn)?;

        let entries_resolved = entries
            .into_iter()
            .map(|(entry, hardest)| CountryLeaderboardEntryResolved {
                rank: entry.rank,
                extremes_rank: entry.extremes_rank,
                country: entry.country,
                level_points: entry.level_points,
                members_count: entry.members_count,
                hardest,
                extremes: entry.extremes,
            })
            .collect::<Vec<_>>();

        let refresh_log: MatviewRefreshLog = matview_refresh_log::table
            .find("aredl_country_leaderboard")
            .first(conn)
            .unwrap_or(MatviewRefreshLog {
                view_name: "aredl_country_leaderboard".into(),
                last_refresh: Utc::now(),
            });

        Ok(Paginated::<Self>::from_data(
            page_query,
            count,
            Self {
                last_refresh: refresh_log.last_refresh,
                data: entries_resolved,
            },
        ))
    }
}
