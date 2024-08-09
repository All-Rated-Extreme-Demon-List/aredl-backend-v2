use std::sync::Arc;
use actix_web::{get, post, patch, HttpResponse, web};
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::{history, packs, Level, LevelPlace, LevelUpdate, ResolvedLevel, records, creators};
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("")]
async fn list(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let levels = web::block(
        || Level::find_all(db)
    ).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[post("", wrap="UserAuth::require(Permission::LevelModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, level: web::Json<LevelPlace>) -> Result<HttpResponse, ApiError> {
    let level = web::block(
        || Level::create(db, level.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::LevelModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<String>,level: web::Json<LevelUpdate>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, id.into_inner().as_str())?;
    let level = web::block(
        move || Level::update(db, level_id, level.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[get("/{id}")]
async fn find(db: web::Data<Arc<DbAppState>>, id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, id.into_inner().as_str())?;
    let level = web::block(
        move || ResolvedLevel::find(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(level))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/levels")
            .service(list)
            .service(create)
            .service(update)
            .service(find)
            .configure(history::init_routes)
            .configure(packs::init_routes)
            .configure(records::init_routes)
            .configure(creators::init_routes)
    );
}