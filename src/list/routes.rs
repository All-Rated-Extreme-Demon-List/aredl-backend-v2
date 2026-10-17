use crate::list::{
    bounty, changelog, leaderboard, levels, packs, packtiers, profile, records, statistics,
    submissions, List,
};
use crate::utils::redirect_prefix;
use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    components(schemas(List)),
    tags(
        (name = "List", description = "AREDL related endpoints. For levels data, refer to [List - Levels.](#get-/api/{list}/levels)")
    ),
    nest(
        (path = "/levels", api=levels::ApiDoc),
        (path = "/leaderboard", api=leaderboard::ApiDoc),
        (path = "/changelog", api=changelog::ApiDoc),
        (path = "/packs", api=packs::ApiDoc),
        (path = "/pack-tiers", api=packtiers::ApiDoc),
        (path = "/profile", api=profile::ApiDoc),
        (path = "/submissions", api=submissions::ApiDoc),
        (path = "/records", api=records::ApiDoc),
        (path = "/statistics", api=statistics::ApiDoc),
        (path = "/bounty-board", api=bounty::ApiDoc),
    ),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{list}")
            .configure(submissions::init_routes)
            .configure(levels::init_routes)
            .configure(changelog::init_routes)
            .configure(packtiers::init_routes)
            .configure(packs::init_routes)
            .configure(leaderboard::init_routes)
            .configure(profile::init_routes)
            .service(redirect_prefix("/clan", "/profile/clan"))
            .service(redirect_prefix("/country", "/profile/country"))
            .configure(records::init_routes)
            .configure(statistics::init_routes)
            .configure(bounty::init_routes),
    );
}
