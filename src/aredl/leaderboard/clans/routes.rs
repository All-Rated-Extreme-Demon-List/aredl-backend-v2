use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::leaderboard::clans::model::{ClansLeaderboardPage, ClansLeaderboardQueryOptions};
use crate::aredl::leaderboard::model::LeaderboardOrder;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};

#[utoipa::path(
    get,
    summary = "Leaderboard - Clans",
    description = "Get the clans leaderboard paginated data. Refreshes hourly",
    tag = "AREDL",
    params(
        ("page" = Option<i64>, Query, description = "The page of the clans leaderboard to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("order" = Option<LeaderboardOrder>, Query, description = "The sorting type to use. Defaults to using points"),
    ),
    responses(
        (status = 200, body = [Paginated<ClansLeaderboardPage>])
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<ClansLeaderboardQueryOptions>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClansLeaderboardPage::find(&mut conn, page_query.into_inner(), options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            ClansLeaderboardPage
        )
    ),
    paths(
        list
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/clans")
            .service(list)
    );
}