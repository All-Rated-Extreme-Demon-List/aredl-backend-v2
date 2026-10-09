use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::list::changelog::ChangelogPage;
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Changelog",
    description = "Get the changelog paginated data.",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        PageQuery<20>,
    ),
    responses(
        (status = 200, body = Paginated<ChangelogPage>),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page_query: web::Query<PageQuery<20>>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        ChangelogPage::find(&mut db.connection()?, *list, page_query.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ChangelogPage)), paths(find_all))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/changelog").service(find_all));
}
