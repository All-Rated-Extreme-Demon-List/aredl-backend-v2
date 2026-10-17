use crate::list::profile::{clan, country, user};
use crate::utils::redirect_route;
use actix_web::web;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(nest(
	(path = "/user", api=user::ApiDoc),
	(path = "/country", api=country::ApiDoc),
	(path = "/clan", api=clan::ApiDoc),
))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/profile")
            .configure(user::init_routes)
            .configure(clan::init_routes)
            .configure(country::init_routes)
            .route("/{id}", redirect_route("/profile", "/profile/user")),
    );
}
