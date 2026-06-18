use crate::app_data::auth::AuthAppState;
use crate::app_data::db::DbAppState;
use crate::auth::token::UserClaims;
use crate::auth::token::{self, check_token_valid};
use crate::error_handler::ApiError;
use actix_http::header;
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, TimeZone, Utc};
use serde::Serialize;
use std::sync::Arc;
use utoipa::{OpenApi, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
struct AuthRefreshResponse {
    /// The new access token to use for authentication. Expires after 30 minutes.
    pub access_token: String,
    /// Timestamp of when the access token expires.
    pub access_expires: DateTime<Utc>,
    /// The new refresh token to use for getting a new access token. Expires after 2 weeks.
    pub refresh_token: Option<String>,
    /// Timestamp of when the refresh token expires.
    pub refresh_expires: Option<DateTime<Utc>>,
}

#[utoipa::path(
    get,
    summary = "[Auth]Refresh auth",
    description = "Get a new access token. If the refresh token is about to expire, will also return a new one.",
    tag = "Authentication",
    responses(
        (status = 200, body = AuthRefreshResponse)
    ),
    security(
        ("refresh_token" = []),
    )
)]
#[get("")]
async fn refresh_auth(
    data: web::Data<Arc<AuthAppState>>,
    db: web::Data<Arc<DbAppState>>,
    req: HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let refresh_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.strip_prefix("Bearer ").unwrap_or("").to_string());

    if refresh_token.is_none() {
        return Err(ApiError::new(400, "No token provided"));
    }

    let decoded_token_claims = token::decode_token(
        refresh_token.unwrap(),
        &data.jwt_decoding_key,
        &["refresh", "initial"],
    )?;

    let decoded_user_claims = token::decode_user_claims(&decoded_token_claims)?;

    let conn = &mut db.connection()?;
    check_token_valid(&decoded_token_claims, &decoded_user_claims, conn)?;

    let user_id = decoded_user_claims.user_id;

    let (access_token, access_expires) = token::create_token(
        UserClaims {
            user_id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::minutes(30),
        "access",
    )?;

    let mut response = serde_json::json!({
        "access_token": access_token,
        "access_expires": access_expires,
    });

    let now = Utc::now();
    let refresh_exp = Utc
        .timestamp_opt(decoded_token_claims.exp as i64, 0)
        .single()
        .ok_or_else(|| ApiError::new(500, "Failed to parse expiration timestamp"))?;

    if refresh_exp - now < Duration::days(2) {
        let (new_refresh_token, refresh_expires) = token::create_token(
            UserClaims {
                user_id,
                is_api_key: false,
            },
            &data.jwt_encoding_key,
            Duration::weeks(2),
            "refresh",
        )?;

        response["refresh_token"] = serde_json::Value::String(new_refresh_token);
        response["refresh_expires"] = serde_json::Value::String(refresh_expires.to_rfc3339());
    }

    Ok(HttpResponse::Ok().json(response))
}

#[derive(OpenApi)]
#[openapi(components(schemas(AuthRefreshResponse)), paths(refresh_auth))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/refresh").service(refresh_auth));
}
