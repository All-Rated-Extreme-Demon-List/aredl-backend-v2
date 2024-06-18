use actix_web::{get, HttpResponse, web};
use crate::aredl::changelog::model::ChangelogEntry;
use crate::error_handler::ApiError;
use crate::page_helper::PageQuery;

#[get("")]
async fn list(page_query: web::Query<PageQuery<20>>) -> Result<HttpResponse, ApiError> {
    let result = web::block(||
        ChangelogEntry::find_all(page_query.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(result))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/changelog")
            .service(list)
    );
}