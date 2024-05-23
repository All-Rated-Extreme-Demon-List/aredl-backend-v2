use actix_web::web;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/packs")
    );
}