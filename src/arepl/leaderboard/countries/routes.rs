use crate::arepl::leaderboard::countries::{
    CountryLeaderboardPage, CountryLeaderboardQueryOptions,
};
use crate::arepl::leaderboard::LeaderboardOrder;
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Leaderboard - Countries",
    description = "Get the countries leaderboard paginated data. Refreshes hourly",
    tag = "AREDL (P)",
    params(
        ("page" = Option<i64>, Query, description = "The page of the countries leaderboard to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points"),
    ),
    responses(
        (status = 200, body = [Paginated<CountryLeaderboardPage>])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<CountryLeaderboardQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        CountryLeaderboardPage::find(&mut conn, page_query.into_inner(), options.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(CountryLeaderboardPage)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/countries").service(list));
}
