use actix_web::{get, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels::history::HistoryLevelFull;
use crate::aredl::levels::LevelId;
use crate::error_handler::ApiError;

#[get("")]
async fn find(id: web::Path<LevelId>) -> Result<HttpResponse, ApiError> {
    let entries = web::block(|| HistoryLevelFull::find(id.into_inner().into())).await??;
    Ok(HttpResponse::Ok().json(entries))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/history")
            .service(find)
    );
}