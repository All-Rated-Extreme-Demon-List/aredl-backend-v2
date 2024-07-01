use actix_web::{get, HttpResponse, post, web};
use crate::aredl::packtiers::model::PackTierResolved;
use crate::aredl::packtiers::{PackTier, PackTierCreate};
use crate::error_handler::ApiError;

#[get("")]
async fn find_all() -> Result<HttpResponse, ApiError> {
    let tiers = web::block(
        || PackTierResolved::find_all()
    ).await??;
    Ok(HttpResponse::Ok().json(tiers))
}

#[post("")]
async fn create(tier: web::Json<PackTierCreate>) -> Result<HttpResponse, ApiError> {
    let tier = web::block(
        || PackTier::create(tier.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(tier))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("pack-tiers")
            .service(find_all)
            .service(create)
    );
}