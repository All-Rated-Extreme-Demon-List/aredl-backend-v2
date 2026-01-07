use crate::app_data::db::DbConnection;
use crate::auth::{permission, Permission};
use crate::page_helper::{PageQuery, Paginated};
use crate::{
    error_handler::ApiError,
    schema::{arepl::submission_stats, users},
    users::{BaseUser, ExtendedBaseUser},
};
use chrono::NaiveDate;
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submission_stats, check_for_backend(Pg))]
pub struct DailyStats {
    pub day: NaiveDate,
    pub reviewer_id: Option<Uuid>,
    pub submitted: i64,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedLeaderboardRow {
    pub moderator: ExtendedBaseUser,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub total: i64,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct ResolvedDailyStats {
    pub date: NaiveDate,
    pub moderator: Option<BaseUser>,
    pub submitted: i64,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DailyStatsPage {
    pub data: Vec<ResolvedDailyStats>,
}

impl ResolvedDailyStats {
    pub fn from_stats_and_user(stats: DailyStats, user: Option<BaseUser>) -> Self {
        Self {
            date: stats.day,
            moderator: user,
            submitted: stats.submitted,
            accepted: stats.accepted,
            denied: stats.denied,
            under_consideration: stats.under_consideration,
        }
    }
}
impl DailyStatsPage {
    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        reviewer_id: Option<Uuid>,
    ) -> Result<Paginated<Self>, ApiError> {
        let build_filtered_query = || {
            let mut q = submission_stats::table
                .left_join(users::table.on(users::id.nullable().eq(submission_stats::reviewer_id)))
                .into_boxed::<Pg>();

            if let Some(ref filter) = reviewer_id {
                q = q.filter(submission_stats::reviewer_id.eq(filter));
            } else {
                q = q.filter(submission_stats::reviewer_id.is_null());
            }
            q
        };

        let count: i64 = build_filtered_query().count().get_result(conn)?;

        let data: Vec<(DailyStats, Option<BaseUser>)> = build_filtered_query()
            .select((DailyStats::as_select(), Option::<BaseUser>::as_select()))
            .order(submission_stats::day.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .load(conn)?;

        Ok(Paginated::from_data(
            page_query,
            count,
            Self {
                data: data
                    .into_iter()
                    .map(|(stats, user)| ResolvedDailyStats::from_stats_and_user(stats, user))
                    .collect(),
            },
        ))
    }
}

pub fn stats_mod_leaderboard(
    conn: &mut DbConnection,
    since: Option<NaiveDate>,
    only_active: bool,
) -> Result<Vec<ResolvedLeaderboardRow>, ApiError> {
    let mut query = submission_stats::table
        .inner_join(users::table.on(users::id.nullable().eq(submission_stats::reviewer_id)))
        .select((DailyStats::as_select(), ExtendedBaseUser::as_select()))
        .into_boxed::<Pg>();

    if let Some(date) = since {
        query = query.filter(submission_stats::day.ge(date));
    }

    let all_rows: Vec<(DailyStats, ExtendedBaseUser)> = query.load(conn)?;

    let rows = all_rows.into_iter().filter_map(|(stats, user)| {
        if only_active {
            match permission::check_permission(conn, user.id, Permission::SubmissionReview) {
                Ok(true) => Some((stats, user)),
                _ => None,
            }
        } else {
            Some((stats, user))
        }
    });

    let acc: HashMap<Uuid, ResolvedLeaderboardRow> =
        rows.into_iter()
            .fold(HashMap::new(), |mut map, (stats, user)| {
                let total_for_day = stats.accepted + stats.denied + stats.under_consideration;

                map.entry(user.id)
                    .and_modify(|row| {
                        row.accepted += stats.accepted;
                        row.denied += stats.denied;
                        row.under_consideration += stats.under_consideration;
                        row.total += total_for_day;
                    })
                    .or_insert_with(|| ResolvedLeaderboardRow {
                        moderator: user,
                        accepted: stats.accepted,
                        denied: stats.denied,
                        under_consideration: stats.under_consideration,
                        total: total_for_day,
                    });
                map
            });

    let mut leaderboard = acc.into_values().collect::<Vec<_>>();
    leaderboard.sort_unstable_by(|a, b| b.total.cmp(&a.total));

    Ok(leaderboard)
}
