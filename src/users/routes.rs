use actix_web::web;
use crate::users::me;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/users")
            .configure(me::init_routes)
    );
}