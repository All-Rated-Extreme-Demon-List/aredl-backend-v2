use crate::aredl::changelog::ChangelogPage;
use crate::cache_control::CacheController;
use crate::app_data::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Changelog",
    description = "Get the changelog paginated data.",
    tag = "AREDL",
    params(
        ("page" = Option<i64>, Query, description = "The page of the changelog to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
    ),
    responses(
        (status = 200, body = [Paginated<ChangelogPage>])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<20>>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || ChangelogPage::find(&mut db.connection()?, page_query.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ChangelogPage)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/changelog").service(list));
}
