use uuid::Uuid;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, JoinOnDsl, NullableExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
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
    pub country_rank: i32,
    pub user_id: Uuid,
    pub country: Option<i32>,
    pub discord_id: Option<String>,
    pub discord_avatar: Option<String>,
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
    pub country_rank: i32,
    pub user: User,
    pub country: Option<i32>,
    pub discord_id: Option<String>,
    pub discord_avatar: Option<String>,
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
pub struct LeaderboardQueryOptions {
    pub name_filter: Option<String>,
    pub country: Option<i32>
}

impl LeaderboardPage {
    pub fn find<const D: i64>(conn: &mut DbConnection, page_query: PageQuery<D>, options: LeaderboardQueryOptions) -> Result<Paginated<Self>, ApiError> {
        let name_filter = match options.name_filter {
            Some(filter) => users::global_name.ilike(filter),
            None => users::global_name.ilike("%".to_string()),
        };
        let selection = (
            LeaderboardEntry::as_select(),
            User::as_select(),
            (aredl_levels::id, aredl_levels::name).nullable()
        );

        let query =
            aredl_user_leaderboard::table
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .filter(name_filter)
                .inner_join(users::table.on(users::id.eq(aredl_user_leaderboard::user_id)))
                .left_join(aredl_levels::table.on(aredl_user_leaderboard::hardest.eq(aredl_levels::id.nullable())));

        let (entries, count) : (Vec<(LeaderboardEntry, User, Option<Level>)>, i64) = match options.country{
            None => {
                let data = query.clone()
                    .order(aredl_user_leaderboard::rank)
                    .select(selection)
                    .load::<(LeaderboardEntry, User, Option<Level>)>(conn)?;
                let count = query.count().first(conn)?;
                Ok::<(Vec<(LeaderboardEntry, User, Option<Level>)>, i64), ApiError>((data, count))
            },
            Some(country) => {
                let query = query
                    .filter(aredl_user_leaderboard::country.eq(country));
                let data = query
                    .clone()
                    .order(aredl_user_leaderboard::country_rank)
                    .select(selection)
                    .load::<(LeaderboardEntry, User, Option<Level>)>(conn)?;
                let count = query.count().first(conn)?;
                Ok::<(Vec<(LeaderboardEntry, User, Option<Level>)>, i64), ApiError>((data, count))
            }
        }?;

        let entries_resolved = entries
            .into_iter()
            .map(|(entry, user, hardest)| LeaderboardEntryResolved {
                rank: entry.rank,
                country_rank: entry.country_rank,
                user,
                country: entry.country,
                discord_id: entry.discord_id,
                discord_avatar: entry.discord_avatar,
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