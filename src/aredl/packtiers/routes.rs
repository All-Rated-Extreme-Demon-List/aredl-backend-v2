use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::packtiers::model::PackTierResolved;
use crate::aredl::packtiers::{PackTier, PackTierCreate, PackTierUpdate};
use crate::auth::{UserAuth, Authenticated, Permission};
use crate::error_handler::ApiError;

#[get("", wrap="UserAuth::load()")]
async fn find_all(authenticated: Option<Authenticated>) -> Result<HttpResponse, ApiError> {
    let tiers = web::block(
        || PackTierResolved::find_all(authenticated.map(|user| user.user_id))
    ).await??;
    Ok(HttpResponse::Ok().json(tiers))
}

#[post("", wrap="UserAuth::require(Permission::PackTierModify)")]
async fn create(tier: web::Json<PackTierCreate>) -> Result<HttpResponse, ApiError> {
    let tier = web::block(
        || PackTier::create(tier.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::PackTierModify)")]
async fn update(id: web::Path<Uuid>, tier: web::Json<PackTierUpdate>) -> Result<HttpResponse, ApiError> {
    let tier = web::block(
        || PackTier::update(id.into_inner(), tier.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::PackTierModify)")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let tier = web::block(
        || PackTier::delete(id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(tier))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("pack-tiers")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete)
    );
}