use std::sync::Arc;
use actix_web::{post, web, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use utoipa::OpenApi;
use crate::db::DbAppState;
use crate::auth::app_state::AuthAppState;
use crate::error_handler::ApiError;
use crate::schema::users;
use crate::auth::token::{self, check_token_valid};

#[utoipa::path(
    post,
	summary = "[Auth]Logout",
	description = "Log out all of the current user's sessions.",
    responses(
        (status = 200)
    ),
    tag = "Authentication",
	security(
		("access_token" = []),
		("refresh_token" = []),
		("api_key" = []),
	)
)]
#[post("")]
pub async fn logout_all(req: HttpRequest, data: web::Data<Arc<AuthAppState>>, db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
	let token = req
		.headers()
		.get(openidconnect::http::header::AUTHORIZATION)
		.and_then(|h| h.to_str().ok())
		.map(|h| h.strip_prefix("Bearer ").unwrap_or("").to_string());

	if token.is_none() {
		return Err(ApiError::new(400, "No token provided"));
	}

	let decoded_token_claims = token::decode_token(
		token.unwrap(),
		&data.jwt_decoding_key,
		&["access", "refresh"]
	)?;

	let decoded_user_claims = token::decode_user_claims(&decoded_token_claims)?;
	let mut conn = db.connection()?;

	check_token_valid(&decoded_token_claims, &decoded_user_claims, &mut conn)?;

    diesel::update(users::table.filter(users::id.eq(decoded_user_claims.user_id)))
        .set(users::access_valid_after.eq(Utc::now().naive_utc()))
        .execute(&mut conn)?;

    Ok(HttpResponse::Ok().json("Logged out all sessions"))
}

#[derive(OpenApi)]
#[openapi(
	paths(
		logout_all,
	)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth/logout-all")
            .service(logout_all)
    );
}