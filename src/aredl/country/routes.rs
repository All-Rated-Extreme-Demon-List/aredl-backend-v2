use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::country::CountryProfileResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    get,
    summary = "Country",
    description = "Get a country's AREDL profile",
    tag = "AREDL",
    params(
        ("id" = Uuid, description = "The country to lookup the data for")
    ),
    responses(
        (status = 200, body = CountryProfileResolved)
    ),
)]
#[get("/{id}")]
async fn find(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        let mut conn = db.connection()?;
        CountryProfileResolved::find(&mut conn, id.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            CountryProfileResolved
        )
    ),
    paths(
        find
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("country")
            .service(find)
    );
}