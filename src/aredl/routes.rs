use actix_web::web;
use crate::aredl::levels;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/aredl")
            .configure(levels::init_routes)
    );
}