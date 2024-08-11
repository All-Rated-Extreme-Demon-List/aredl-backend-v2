use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use crate::auth::{UserAuth, Authenticated};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::me::model::User;

#[get("", wrap="UserAuth::load()")]
async fn find(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let user = web::block(
        move || {
            let conn = &mut db.connection()?;
            User::find(conn, authenticated.user_id)
        }
    ).await??;
    Ok(HttpResponse::Ok().json(user))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/@me")
            .service(find)
    );
}