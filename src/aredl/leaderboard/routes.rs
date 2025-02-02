use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::leaderboard::model::{LeaderboardPage, LeaderboardQueryOptions, LeaderboardOrder};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};

#[utoipa::path(
    get,
    summary = "Leaderboard",
    description = "Get a leaderboard paginated data. Refreshes hourly",
    tag = "AREDL",
    params(
        ("page" = Option<i64>, Query, description = "The page of the leaderboard to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("name_filter" = Option<String>, Query, description = "Search filter to apply. Uses the SQL LIKE operator syntax."),
        ("country_filter" = Option<i32>, Query, description = "The country filter to apply. Uses the ISO 3166-1 country codes"),
        ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points (with packs)"),
    ),
    responses(
        (status = 200, body = [Paginated<LeaderboardPage>])
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<LeaderboardQueryOptions>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        LeaderboardPage::find(&mut conn, page_query.into_inner(), options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
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
            .service(list)
    );
}