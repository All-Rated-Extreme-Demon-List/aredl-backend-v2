use std::sync::Arc;
use actix_web::{get, web, HttpResponse};
use serde::{Deserialize, Serialize};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::PageQuery;
use crate::users::{me, names, User, UserListQueryOptions};

#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<UserListQueryOptions>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        User::find(&mut conn, page_query.into_inner(), options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/users")
            .service(list)
            .configure(me::init_routes)
            .configure(names::init_routes)
    );
}