use actix_web::web;
use crate::aredl::levels;

pub fn init_routes(config: &mut web::ServiceConfig) {
    levels::init_routes(config);
}