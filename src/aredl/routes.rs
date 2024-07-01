use actix_web::web;
use crate::aredl::{changelog, levels, packtiers};

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/aredl")
            .configure(levels::init_routes)
            .configure(changelog::init_routes)
            .configure(packtiers::init_routes)
    );
}