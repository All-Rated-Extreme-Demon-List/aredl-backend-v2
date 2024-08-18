use actix_web::web;
use crate::aredl::{changelog, leaderboard, levels, packs, packtiers, profile};

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/aredl")
            .configure(levels::init_routes)
            .configure(changelog::init_routes)
            .configure(packtiers::init_routes)
            .configure(packs::init_routes)
            .configure(leaderboard::init_routes)
            .configure(profile::init_routes)
    );
}