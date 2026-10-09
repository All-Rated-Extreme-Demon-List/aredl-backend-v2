use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::list::leaderboard::clans::{ClansLeaderboardPage, ClansLeaderboardQueryOptions};
use crate::list::leaderboard::LeaderboardOrder;
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Leaderboard - Clans",
    description = "Get the clans leaderboard paginated data. Refreshes hourly",
    tag = "List",
    params(
        PageQuery<100>,
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points"),
        ("name_filter" = Option<String>, Query, description = "Search filter to apply. Uses the SQL LIKE operator syntax."),
    ),
    responses(
        (status = 200, body = Paginated<ClansLeaderboardPage>),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<ClansLeaderboardQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        ClansLeaderboardPage::find(
            &mut db.connection()?,
            *list,
            page_query.into_inner(),
            options.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ClansLeaderboardPage)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/clans").service(list));
}
