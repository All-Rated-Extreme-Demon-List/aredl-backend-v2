use crate::list::List;
use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    list::statistics::submissions::daily::{
        stats_mod_leaderboard, DailyStatsPage, ResolvedLeaderboardRow,
    },
    page_helper::{PageQuery, Paginated},
};
use actix_web::{get, web, HttpResponse};
use chrono::NaiveDate;
use serde::Deserialize;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Deserialize, ToSchema)]
pub struct StatsQuery {
    pub reviewer_id: Option<Uuid>,
    pub level_id: Option<Uuid>,
}

#[utoipa::path(
    get,
    summary = "[Staff]Get submission statistics",
    description = "Get per-day submission statistics, optionally filtered by reviewer or level.",
    tag = "List - Statistics",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        PageQuery<31, 3650>,
        ("reviewer_id" = Option<Uuid>, Query, description = "Filter for a specific reviewer"),
        ("level_id" = Option<Uuid>, Query, description = "Filter for a specific level")
    ),
    responses(
        (status = 200, body = Paginated<DailyStatsPage>),
        (status = 400, description = "You can not filter by both reviewer and level at the same time", body = ErrorResponse),
    ),
    security(("bearer_token" = ["SubmissionSeeStatistics"])),
)]
#[get("", wrap = "UserAuth::require(Permission::SubmissionSeeStatistics)")]
pub async fn stats(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page: web::Query<PageQuery<31, 3650>>,
    query: web::Query<StatsQuery>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let stats = web::block(move || {
        let query = query.into_inner();
        DailyStatsPage::find(
            &mut db.connection()?,
            *list,
            page.into_inner(),
            query.reviewer_id,
            query.level_id,
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(stats))
}

#[derive(Deserialize, ToSchema)]
pub struct LeaderboardQuery {
    pub since: Option<NaiveDate>,
    pub until: Option<NaiveDate>,
    pub reviewer_id: Option<Uuid>,
    pub only_active: Option<bool>,
    pub include_hidden_reviewers: Option<bool>,
}

#[utoipa::path(
    get,
    summary = "[Staff]Reviewer leaderboard",
    description = "List reviewers ranked by number of checked submissions.",
    tag = "List - Statistics",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("since" = Option<NaiveDate>, Query, description = "Only include data since this date"),
        ("until" = Option<NaiveDate>, Query, description = "Only include data until this date"),
        ("reviewer_id" = Option<Uuid>, Query, description = "Filter for a specific reviewer"),
        ("only_active" = Option<bool>, Query, description = "Whether or not to exclude reviewers that aren't staff anymore"),
        ("include_hidden_reviewers" = Option<bool>, Query, description = "Whether to include hidden reviewers in the results. Requires `ReviewersAudit`; otherwise forced to false."),
    ),
    responses(
        (status = 200, body = [ResolvedLeaderboardRow]),
    ),
    security(("bearer_token" = ["SubmissionSeeStatistics"])),
)]
#[get(
    "/leaderboard",
    wrap = "UserAuth::require(Permission::SubmissionSeeStatistics)"
)]
pub async fn leaderboard_route(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    query: web::Query<LeaderboardQuery>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let data = web::block(move || {
        stats_mod_leaderboard(
            &mut db.connection()?,
            *list,
            &query.into_inner(),
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(data))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(DailyStatsPage, ResolvedLeaderboardRow, StatsQuery, LeaderboardQuery)),
    paths(stats, leaderboard_route)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/daily")
            .service(stats)
            .service(leaderboard_route),
    );
}
