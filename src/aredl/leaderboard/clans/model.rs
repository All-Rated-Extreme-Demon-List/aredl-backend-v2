use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{BoxableExpression, ExpressionMethods, PgTextExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::expression::expression_types::NotSelectable;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use crate::custom_schema::aredl_clans_leaderboard;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::aredl::levels::BaseLevel;
use crate::aredl::leaderboard::LeaderboardOrder;
use crate::schema::{aredl_levels, clans};
use crate::clans::Clan;

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=aredl_clans_leaderboard, check_for_backend(Pg))]
pub struct ClansLeaderboardEntry {
    pub rank: i32,
    pub extremes_rank: i32,
    pub clan_id: Uuid,
    pub level_points: i32,
	pub members_count: i32,
    pub hardest: Option<Uuid>,
    pub extremes: i32
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
    pub extremes: i32
}

#[derive(Serialize, Debug, ToSchema)]
pub struct ClansLeaderboardPage {
    /// List of leaderboard entries.
    pub data: Vec<ClansLeaderboardEntryResolved>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ClansLeaderboardQueryOptions {
    pub order: Option<LeaderboardOrder>,
    pub name_filter: Option<String>,
}

impl ClansLeaderboardPage {
    pub fn find<const D: i64>(conn: &mut DbConnection, page_query: PageQuery<D>, options: ClansLeaderboardQueryOptions) -> Result<Paginated<Self>, ApiError> {
        let selection = (
            ClansLeaderboardEntry::as_select(),
            Clan::as_select(),
            (aredl_levels::id, aredl_levels::name).nullable(),
        );

        let order = options.order.unwrap_or(LeaderboardOrder::TotalPoints);

        let ordering: Box< dyn BoxableExpression<_, _, SqlType = NotSelectable>> =
            match order {
                LeaderboardOrder::TotalPoints => Box::new(aredl_clans_leaderboard::rank.asc()),
                LeaderboardOrder::ExtremeCount => Box::new(aredl_clans_leaderboard::extremes_rank.asc()),
				LeaderboardOrder::RawPoints => Box::new(aredl_clans_leaderboard::rank.asc()) 
            };

		let mut query = aredl_clans_leaderboard::table
            .inner_join(clans::table.on(clans::id.eq(aredl_clans_leaderboard::clan_id)))
            .left_join(aredl_levels::table.on(
                aredl_clans_leaderboard::hardest.eq(aredl_levels::id.nullable()),
            ))
            .into_boxed();

        if let Some(ref filter) = options.name_filter {
            query = query.filter(clans::global_name.ilike(filter));
        }

        let entries = query
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .order((ordering, aredl_clans_leaderboard::clan_id.asc()))
            .select(selection)
            .load::<(ClansLeaderboardEntry, Clan, Option<BaseLevel>)>(conn)?;

        let mut count_query = aredl_clans_leaderboard::table
            .inner_join(clans::table.on(clans::id.eq(aredl_clans_leaderboard::clan_id)))
            .left_join(aredl_levels::table.on(
                aredl_clans_leaderboard::hardest.eq(aredl_levels::id.nullable()),
            ))
            .into_boxed();

        if let Some(ref filter) = options.name_filter {
            count_query = count_query.filter(clans::global_name.ilike(filter));
        }
        
        let count = count_query
            .count()
            .get_result(conn)?;

        let entries_resolved = entries
            .into_iter()
            .map(|(entry, clan, hardest)| ClansLeaderboardEntryResolved {
                rank: entry.rank,
                extremes_rank: entry.extremes_rank,
				clan,
                level_points: entry.level_points,
				members_count: entry.members_count,
                hardest,
                extremes: entry.extremes
            })
            .collect::<Vec<_>>();

        Ok(Paginated::<Self>::from_data(page_query, count, Self {
            data: entries_resolved
        }))
    }
}