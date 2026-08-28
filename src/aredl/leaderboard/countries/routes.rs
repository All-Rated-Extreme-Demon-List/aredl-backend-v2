use crate::app_data::db::DbAppState;
use crate::aredl::leaderboard::countries::{
    CountryLeaderboardPage, CountryLeaderboardQueryOptions,
};
use crate::aredl::leaderboard::LeaderboardOrder;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Leaderboard - Countries",
    description = "Get the countries leaderboard data. Refreshes hourly",
    tag = "AREDL",
    params(
       ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points"),
    ),
    responses(
        (status = 200, body = [CountryLeaderboardPage])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    options: web::Query<CountryLeaderboardQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        CountryLeaderboardPage::find_all(&mut db.connection()?, options.into_inner())
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
