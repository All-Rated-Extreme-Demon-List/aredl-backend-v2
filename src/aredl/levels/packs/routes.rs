use actix_web::{get, HttpResponse, web};
use crate::aredl::levels::LevelId;
use crate::aredl::levels::packs::PackResolved;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(level_id: web::Path<LevelId>) -> Result<HttpResponse, ApiError> {
    let packs = web::block(
        || PackResolved::find_all(level_id.into_inner().into())
    ).await??;
    Ok(HttpResponse::Ok().json(packs))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/packs")
            .service(find_all)
    );
}