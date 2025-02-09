use actix_web::web;
use utoipa::OpenApi;
use crate::aredl::{changelog, leaderboard, levels, packs, packtiers, profile, country, clan};
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL", description = "AREDL related endpoints. For list/levels data, refer to [AREDL - Levels.](#get-/api/aredl/levels)")
    ),
    nest(
        (path = "/levels", api=levels::ApiDoc),
        (path = "/leaderboard", api=leaderboard::ApiDoc),
        (path = "/changelog", api=changelog::ApiDoc),
        (path = "/packs", api=packs::ApiDoc),
        (path = "/pack-tiers", api=packtiers::ApiDoc),
        (path = "/profile", api=profile::ApiDoc),
        (path = "/country", api=country::ApiDoc),
        (path = "/clan", api=clan::ApiDoc)
    ),
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/aredl")
            .configure(levels::init_routes)
            .configure(changelog::init_routes)
            .configure(packtiers::init_routes)
            .configure(packs::init_routes)
            .configure(leaderboard::init_routes)
            .configure(profile::init_routes)
            .configure(country::init_routes)
            .configure(clan::init_routes)
    );
}