use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::changelog::model::ChangelogPage;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};

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
#[get("")]
async fn list(db: web::Data<Arc<DbAppState>>, page_query: web::Query<PageQuery<20>>) -> Result<HttpResponse, ApiError> {
    let result = web::block(||
        ChangelogPage::find(db, page_query.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            ChangelogPage,
        )
    ),
    paths(
        list
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/changelog")
            .service(list)
    );
}