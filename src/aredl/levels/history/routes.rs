use actix_web::{get, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels::history::HistoryLevelFull;
use crate::error_handler::ApiError;

#[get("/aredl/levels/{id}/history")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let entries = web::block(|| HistoryLevelFull::find(id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(entries))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(find);
}