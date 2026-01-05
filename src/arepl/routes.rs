use crate::arepl::{
    changelog, clan, country, leaderboard, levels, packs, packtiers, profile, records, statistics,
    submissions,
};
use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL (P)", description = "AREDL Platformer related endpoints. For list/levels data, refer to [AREDL Platformer - Levels.](#get-/api/arepl/levels)")
    ),
    nest(
        (path = "/levels", api=levels::ApiDoc),
        (path = "/leaderboard", api=leaderboard::ApiDoc),
        (path = "/changelog", api=changelog::ApiDoc),
        (path = "/packs", api=packs::ApiDoc),
        (path = "/pack-tiers", api=packtiers::ApiDoc),
        (path = "/profile", api=profile::ApiDoc),
        (path = "/country", api=country::ApiDoc),
        (path = "/clan", api=clan::ApiDoc),
        (path = "/submissions", api=submissions::ApiDoc),
        (path = "/records", api=records::ApiDoc),
        (path = "/statistics", api=statistics::ApiDoc)
    ),
)]

pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/arepl")
            .configure(submissions::init_routes)
            .configure(levels::init_routes)
            .configure(changelog::init_routes)
            .configure(packtiers::init_routes)
            .configure(packs::init_routes)
            .configure(leaderboard::init_routes)
            .configure(profile::init_routes)
            .configure(country::init_routes)
            .configure(clan::init_routes)
            .configure(records::init_routes)
            .configure(statistics::init_routes),
    );
}
