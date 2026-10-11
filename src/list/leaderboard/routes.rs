use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::list::leaderboard::{clans, countries};
use crate::list::leaderboard::{LeaderboardOrder, LeaderboardPage, LeaderboardQueryOptions};
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Leaderboard",
    description = "Get a leaderboard paginated data. Refreshes hourly",
    tag = "List",
    params(
        PageQuery<100>,
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("name_filter" = Option<String>, Query, description = "Search filter to apply. Uses the SQL LIKE operator syntax."),
        ("clan_filter" = Option<Uuid>, Query, description = "The clan filter to apply. Uses the internal clan UUID"),
        ("country_filter" = Option<i32>, Query, description = "The country filter to apply. Uses the ISO 3166-1 country codes"),
        ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points (with packs)"),
    ),
    responses(
        (status = 200, body = Paginated<LeaderboardPage>),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<LeaderboardQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        LeaderboardPage::find(
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
#[openapi(
    nest(
        (path = "/countries", api = countries::ApiDoc),
        (path = "/clans", api = clans::ApiDoc)
    ),
    components(
        schemas(
            LeaderboardPage,
            LeaderboardOrder
        )
    ),
    paths(
        list
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/leaderboard")
            .configure(countries::init_routes)
            .configure(clans::init_routes)
            .service(list),
    );
}
