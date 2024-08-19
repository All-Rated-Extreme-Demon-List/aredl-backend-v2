use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::names::RoleResolved;

#[get("")]
async fn list(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let roles = web::block(move || {
        let mut conn = db.connection()?;
        RoleResolved::find_all(&mut conn)
    }).await??;
    Ok(HttpResponse::Ok().json(roles))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/names")
            .service(list)
    );
}