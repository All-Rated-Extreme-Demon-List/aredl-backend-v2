use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::profile::ProfileResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("/{id}")]
async fn find(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        let mut conn = db.connection()?;
        ProfileResolved::find(&mut conn, id.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(profile))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("profile")
            .service(find)
    );
}