use actix_web::{get, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels::packs::Pack;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(level_id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let packs = web::block(
        || Pack::find_all(level_id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(packs))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/packs")
            .service(find_all)
    );
}