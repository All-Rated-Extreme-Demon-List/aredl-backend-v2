use actix_http::header;
use actix_web::{post, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

use crate::app_data::auth::AuthAppState;
use crate::app_data::db::DbAppState;
use crate::auth::token::UserClaims;
use crate::auth::token::{self, check_token_valid};
use crate::error_handler::ApiError;

#[derive(Debug, Deserialize)]
pub struct ApiKeyOptions {
    pub lifetime_minutes: i64,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct ApiKeyResponse {
    /// The generated API key token.
    pub api_key: String,
    /// Timestamp when the token expires.
    pub expires: DateTime<Utc>,
}

#[utoipa::path(
    post,
	summary = "[Auth]Create API key",
    description = "Generate a new API Key token for the authenticated user, with the given lifetime.",
	params(
		("lifetime_minutes" = i64, Query, description = "Lifetime of the API key token to generate, in minutes.")
	),
    responses(
        (status = 200, body = ApiKeyResponse)
    ),
    tag = "Authentication",
	security(
		("access_token" = []),
		("api_key" = []),
	)
)]
#[post("")]
pub async fn create_api_key(
    req: HttpRequest,
    data: web::Data<Arc<AuthAppState>>,
    options: web::Query<ApiKeyOptions>,
    db: web::Data<Arc<DbAppState>>,
) -> Result<HttpResponse, ApiError> {
    let access_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.strip_prefix("Bearer ").unwrap_or("").to_string());

    if access_token.is_none() {
        return Err(ApiError::new(400, "No token provided"));
    }

    let lifetime_minutes = options.lifetime_minutes;

    let response = web::block(move || {
        let decoded_token_claims =
            token::decode_token(access_token.unwrap(), &data.jwt_decoding_key, &["access"])?;

        let decoded_user_claims = token::decode_user_claims(&decoded_token_claims)?;

        check_token_valid(
            &decoded_token_claims,
            &decoded_user_claims,
            &mut db.connection()?,
        )?;

        let user_id = decoded_user_claims.user_id;

        let lifetime = Duration::minutes(lifetime_minutes);

        if lifetime > Duration::days(365) {
            return Err(ApiError::new(
                400,
                "API key lifetime cannot exceed 1 year (525600 minutes)",
            ));
        }

        let (api_key, expires) = token::create_token(
            UserClaims {
                user_id: user_id,
                is_api_key: true,
            },
            &data.jwt_encoding_key,
            lifetime,
            "access",
        )?;

        Ok(ApiKeyResponse { api_key, expires })
    })
    .await??;

    Ok(HttpResponse::Ok().json(response))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ApiKeyResponse,)), paths(create_api_key,))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/auth/api-key").service(create_api_key));
}
