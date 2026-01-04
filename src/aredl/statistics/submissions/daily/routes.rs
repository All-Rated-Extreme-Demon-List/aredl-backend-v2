use crate::{
    app_data::db::DbAppState,
    aredl::statistics::submissions::daily::{
        stats_mod_leaderboard, DailyStatsPage, ResolvedLeaderboardRow,
    },
    auth::{Permission, UserAuth},
    error_handler::ApiError,
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
}

#[utoipa::path(
    get,
    summary = "[Staff]Get submission statistics",
    description = "Get per-day submission statistics, optionally filtered by moderator.",
    tag = "AREDL - Statistics",
    params(
        ("page" = Option<i64>, Query, description = "The page to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("reviewer_id" = Option<Uuid>, Query, description = "Filter for a specific moderator")
    ),
    responses((status = 200, body = Paginated<DailyStatsPage>)),
    security(("access_token" = ["SubmissionReview"]), ("api_key" = ["SubmissionReview"]))
)]
#[get("", wrap = "UserAuth::require(Permission::SubmissionReview)")]
pub async fn stats(
    db: web::Data<Arc<DbAppState>>,
    page: web::Query<PageQuery<20>>,
    query: web::Query<StatsQuery>,
) -> Result<HttpResponse, ApiError> {
    let stats = web::block(move || {
        DailyStatsPage::find(
            &mut db.connection()?,
            page.into_inner(),
            query.into_inner().reviewer_id,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(stats))
}

#[derive(Deserialize, ToSchema)]
pub struct LeaderboardQuery {
    pub since: Option<NaiveDate>,
    pub only_active: Option<bool>,
}

#[utoipa::path(
    get,
    summary = "[Staff]Moderator leaderboard",
    description = "List moderators ranked by number of checked submissions.",
    tag = "AREDL - Statistics",
    params(
        ("since" = Option<NaiveDate>, Query, description = "Only include data since this date"),
        ("only_active" = Option<bool>, Query, description = "Whether or not to exclude moderators that aren't staff anymore"),
    ),
    responses((status = 200, body = [ResolvedLeaderboardRow])),
    security(("access_token" = ["SubmissionReview"]), ("api_key" = ["SubmissionReview"]))
)]
#[get(
    "/leaderboard",
    wrap = "UserAuth::require(Permission::SubmissionReview)"
)]
pub async fn leaderboard_route(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LeaderboardQuery>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();

    let data = web::block(move || {
        stats_mod_leaderboard(
            &mut db.connection()?,
            query.since,
            query.only_active.unwrap_or(false),
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
