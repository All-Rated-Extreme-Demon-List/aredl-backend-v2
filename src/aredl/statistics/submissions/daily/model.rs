use crate::app_data::db::DbConnection;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::aredl::statistics::submissions::daily::routes::LeaderboardQuery;
use crate::auth::Authenticated;
use crate::page_helper::{PageQuery, Paginated};
use crate::roles::ReviewerVisibility;
use crate::{
    error_handler::ApiError,
    schema::{
        aredl::{
            levels, submission_daily_level_stats, submission_daily_reviewer_stats,
            submission_daily_total_stats,
        },
        users,
    },
    users::{BaseUser, ExtendedBaseUser},
};
use chrono::NaiveDate;
use diesel::pg::Pg;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submission_daily_total_stats, check_for_backend(Pg))]
pub struct TotalDailyStats {
    pub day: NaiveDate,
    pub submitted: i64,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub reviewed: i64,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submission_daily_reviewer_stats, check_for_backend(Pg))]
pub struct ReviewerDailyStats {
    pub day: NaiveDate,
    pub reviewer_id: Uuid,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub reviewed: i64,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = submission_daily_level_stats, check_for_backend(Pg))]
pub struct LevelDailyStats {
    pub day: NaiveDate,
    pub level_id: Uuid,
    pub submitted: i64,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub reviewed: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ResolvedLeaderboardRow {
    pub reviewer: ExtendedBaseUser,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub reviewed: i64,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
pub struct ResolvedDailyStats {
    pub date: NaiveDate,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewer: Option<BaseUser>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<ExtendedBaseLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub submitted: Option<i64>,
    pub accepted: i64,
    pub denied: i64,
    pub under_consideration: i64,
    pub reviewed: i64,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct DailyStatsPage {
    pub data: Vec<ResolvedDailyStats>,
}

impl ResolvedDailyStats {
    pub fn from_total_stats(stats: &TotalDailyStats) -> Self {
        Self {
            date: stats.day,
            reviewer: None,
            level: None,
            submitted: Some(stats.submitted),
            accepted: stats.accepted,
            denied: stats.denied,
            under_consideration: stats.under_consideration,
            reviewed: stats.reviewed,
        }
    }

    pub fn from_reviewer_stats(stats: &ReviewerDailyStats, user: BaseUser) -> Self {
        Self {
            date: stats.day,
            reviewer: Some(user),
            level: None,
            submitted: None,
            accepted: stats.accepted,
            denied: stats.denied,
            under_consideration: stats.under_consideration,
            reviewed: stats.reviewed,
        }
    }

    pub fn from_level_stats(stats: &LevelDailyStats, level: ExtendedBaseLevel) -> Self {
        Self {
            date: stats.day,
            reviewer: None,
            level: Some(level),
            submitted: Some(stats.submitted),
            accepted: stats.accepted,
            denied: stats.denied,
            under_consideration: stats.under_consideration,
            reviewed: stats.reviewed,
        }
    }
}

impl DailyStatsPage {
    pub fn find<const D: i64, const M: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D, M>,
        reviewer_id: Option<Uuid>,
        level_id: Option<Uuid>,
        authenticated: &Authenticated,
    ) -> Result<Paginated<Self>, ApiError> {
        match (reviewer_id, level_id) {
            (Some(_), Some(_)) => Err(ApiError::BadRequest(
                "You can not filter by both reviewer and level at the same time",
            )),
            (Some(reviewer_filter), None) => {
                let visibility = ReviewerVisibility::new(conn, authenticated)?;

                if !visibility.can_see_stats(reviewer_filter, false) {
                    return Ok(Paginated::from_data(
                        page_query,
                        0,
                        Self { data: Vec::new() },
                    ));
                }

                let build_filtered_query = || {
                    submission_daily_reviewer_stats::table
                        .inner_join(
                            users::table
                                .on(users::id.eq(submission_daily_reviewer_stats::reviewer_id)),
                        )
                        .filter(submission_daily_reviewer_stats::reviewer_id.eq(reviewer_filter))
                        .into_boxed::<Pg>()
                };

                let count: i64 = build_filtered_query().count().get_result(conn)?;

                let data: Vec<(ReviewerDailyStats, BaseUser)> = build_filtered_query()
                    .select((ReviewerDailyStats::as_select(), BaseUser::as_select()))
                    .order(submission_daily_reviewer_stats::day.desc())
                    .limit(page_query.per_page())
                    .offset(page_query.offset())
                    .load(conn)?;

                Ok(Paginated::from_data(
                    page_query,
                    count,
                    Self {
                        data: data
                            .into_iter()
                            .map(|(stats, user)| {
                                ResolvedDailyStats::from_reviewer_stats(&stats, user)
                            })
                            .collect(),
                    },
                ))
            }
            (None, Some(level_filter)) => {
                let build_filtered_query = || {
                    submission_daily_level_stats::table
                        .inner_join(
                            levels::table.on(levels::id.eq(submission_daily_level_stats::level_id)),
                        )
                        .filter(submission_daily_level_stats::level_id.eq(level_filter))
                        .into_boxed::<Pg>()
                };

                let count: i64 = build_filtered_query().count().get_result(conn)?;

                let data: Vec<(LevelDailyStats, ExtendedBaseLevel)> = build_filtered_query()
                    .select((LevelDailyStats::as_select(), ExtendedBaseLevel::as_select()))
                    .order(submission_daily_level_stats::day.desc())
                    .limit(page_query.per_page())
                    .offset(page_query.offset())
                    .load(conn)?;

                Ok(Paginated::from_data(
                    page_query,
                    count,
                    Self {
                        data: data
                            .into_iter()
                            .map(|(stats, level)| {
                                ResolvedDailyStats::from_level_stats(&stats, level)
                            })
                            .collect(),
                    },
                ))
            }
            (None, None) => {
                let count: i64 = submission_daily_total_stats::table
                    .count()
                    .get_result(conn)?;

                let data: Vec<TotalDailyStats> = submission_daily_total_stats::table
                    .select(TotalDailyStats::as_select())
                    .order(submission_daily_total_stats::day.desc())
                    .limit(page_query.per_page())
                    .offset(page_query.offset())
                    .load(conn)?;

                Ok(Paginated::from_data(
                    page_query,
                    count,
                    Self {
                        data: data
                            .into_iter()
                            .map(|stats| ResolvedDailyStats::from_total_stats(&stats))
                            .collect(),
                    },
                ))
            }
        }
    }
}

pub fn stats_mod_leaderboard(
    conn: &mut DbConnection,
    options: &LeaderboardQuery,
    authenticated: &Authenticated,
) -> Result<Vec<ResolvedLeaderboardRow>, ApiError> {
    let visibility = ReviewerVisibility::new(conn, authenticated)?;

    let mut query = submission_daily_reviewer_stats::table
        .inner_join(users::table.on(users::id.eq(submission_daily_reviewer_stats::reviewer_id)))
        .select((
            ReviewerDailyStats::as_select(),
            ExtendedBaseUser::as_select(),
        ))
        .filter(submission_daily_reviewer_stats::reviewed.gt(0))
        .into_boxed::<Pg>();

    if let Some(date) = options.since {
        query = query.filter(submission_daily_reviewer_stats::day.ge(date));
    }

    if let Some(date) = options.until {
        query = query.filter(submission_daily_reviewer_stats::day.le(date));
    }

    if let Some(reviewer_id) = options.reviewer_id {
        query = query.filter(submission_daily_reviewer_stats::reviewer_id.eq(reviewer_id));
    }

    if !visibility.can_see_other_stats {
        query =
            query.filter(submission_daily_reviewer_stats::reviewer_id.eq(authenticated.user_id));
    }

    let all_rows: Vec<(ReviewerDailyStats, ExtendedBaseUser)> = query.load(conn)?;

    let rows = all_rows.into_iter().filter(|(_, user)| {
        if options.only_active.unwrap_or(false) && !visibility.is_reviewer(user.id) {
            return false;
        }

        visibility.can_see_stats(user.id, !options.include_hidden_reviewers.unwrap_or(false))
    });

    let acc: HashMap<Uuid, ResolvedLeaderboardRow> =
        rows.into_iter()
            .fold(HashMap::new(), |mut map, (stats, user)| {
                map.entry(user.id)
                    .and_modify(|row| {
                        row.accepted += stats.accepted;
                        row.denied += stats.denied;
                        row.under_consideration += stats.under_consideration;
                        row.reviewed += stats.reviewed;
                    })
                    .or_insert_with(|| ResolvedLeaderboardRow {
                        reviewer: user,
                        accepted: stats.accepted,
                        denied: stats.denied,
                        under_consideration: stats.under_consideration,
                        reviewed: stats.reviewed,
                    });
                map
            });

    let mut leaderboard = acc.into_values().collect::<Vec<_>>();
    leaderboard.sort_unstable_by_key(|b| std::cmp::Reverse(b.reviewed));

    Ok(leaderboard)
}
