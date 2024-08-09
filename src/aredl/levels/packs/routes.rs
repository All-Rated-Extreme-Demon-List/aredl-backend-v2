use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::aredl::levels::packs::PackResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let packs = web::block(
        move || PackResolved::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(packs))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/packs")
            .service(find_all)
    );
}