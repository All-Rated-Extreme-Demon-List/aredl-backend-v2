use std::sync::Arc;

use actix_http::header;
use actix_web::{get, web, HttpRequest, HttpResponse};
use chrono::{DateTime, Duration, TimeZone, Utc};
use diesel::prelude::*;
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreJsonWebKeySet};
use openidconnect::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenUrl,
};

use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::app_data::auth::{AuthAppState, DiscordClient};
use crate::app_data::db::{DbAppState, DbConnection};
use crate::auth::token::UserClaims;
use crate::auth::token::{self, check_token_valid};
use crate::error_handler::ApiError;
use crate::get_secret;
use crate::schema::{oauth_requests, permissions, roles, user_roles};
use crate::users::{Role, User, UserUpsert};

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: AuthorizationCode,
    state: String,
}

#[derive(Serialize, Deserialize, Insertable, Queryable, Selectable)]
#[diesel(table_name=oauth_requests)]
struct OAuthRequestData {
    pub csrf_state: String,
    pub pkce_verifier: String,
    pub callback: Option<String>,
}

impl OAuthRequestData {
    fn find(conn: &mut DbConnection, csrf_state: &str) -> Result<Self, ApiError> {
        let request_data = oauth_requests::table
            .filter(oauth_requests::csrf_state.eq(csrf_state))
            .select(OAuthRequestData::as_select())
            .first::<OAuthRequestData>(conn)?;
        Ok(request_data)
    }

    fn delete(conn: &mut DbConnection, csrf_state: &str) -> Result<(), ApiError> {
        diesel::delete(oauth_requests::table)
            .filter(oauth_requests::csrf_state.eq(csrf_state))
            .execute(conn)?;
        Ok(())
    }

    fn create(conn: &mut DbConnection, data: Self) -> Result<(), ApiError> {
        diesel::insert_into(oauth_requests::table)
            .values(data)
            .execute(conn)?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, ToSchema)]
struct DiscordUser {
    pub id: String,
    pub username: String,
    pub global_name: Option<String>,
    pub avatar: Option<String>,
    pub banner: Option<String>,
    pub accent_color: Option<i32>,
}

impl From<DiscordUser> for UserUpsert {
    fn from(user: DiscordUser) -> Self {
        UserUpsert {
            username: user.username.clone(),
            global_name: user.global_name.unwrap_or(user.username),
            discord_id: Some(user.id),
            placeholder: false,
            country: None,
            discord_avatar: user.avatar,
            discord_banner: user.banner,
            discord_accent_color: user.accent_color,
            last_discord_avatar_update: Some(Utc::now().naive_utc()),
        }
    }
}

#[derive(Debug, Serialize, ToSchema)]
struct AuthResponse {
    /// The access token to use for authentication. Expires after 30 minutes.
    pub access_token: String,
    /// Timestamp of when the access token expires.
    pub access_expires: DateTime<Utc>,
    /// The refresh token to use for getting a new access token. Expires after 2 weeks.
    pub refresh_token: String,
    /// Timestamp of when the refresh token expires.
    pub refresh_expires: DateTime<Utc>,
    /// The user data of the authenticated user.
    #[serde(flatten)]
    pub user: User,
    /// The permissions scopes the user has access to.
    pub scopes: Vec<String>,
    /// The roles the user has.
    pub roles: Vec<Role>,
}

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
#[derive(Debug, Serialize, Deserialize)]
struct AuthOptions {
    pub callback: Option<String>,
}

#[utoipa::path(
    get,
    summary = "Login with Discord",
    description = "Used to authenticate with discord. Creates a Discord OAuth2 flow, which then redirects to [Discord Callback](#get-/api/auth/discord/callback)",
    tag = "Authentication",
    responses(
        (status = 302)
    ),
)]
#[get("")]
async fn discord_auth(
    data: web::Data<Arc<AuthAppState>>,
    db: web::Data<Arc<DbAppState>>,
    options: web::Query<AuthOptions>,
) -> Result<HttpResponse, ApiError> {
    let authorize_url = web::block(move || {
        let client = &data.discord_client;
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let (authorize_url, csrf_state, _) = client
            .authorize_url(
                CoreAuthenticationFlow::AuthorizationCode,
                CsrfToken::new_random,
                Nonce::new_random,
            )
            .add_scope(Scope::new("identify".to_string()))
            .set_pkce_challenge(pkce_challenge.clone())
            .url();

        OAuthRequestData::create(
            &mut db.connection()?,
            OAuthRequestData {
                csrf_state: csrf_state.secret().clone(),
                pkce_verifier: pkce_verifier.secret().to_string(),
                callback: options.callback.clone(),
            },
        )?;
        Ok::<_, ApiError>(authorize_url)
    })
    .await??;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url.to_string()))
        .finish())
}

#[utoipa::path(
    get,
    summary = "Discord Callback",
    description = "End of the discord Oauth2 flow, returns the authenticated user data",
    tag = "Authentication",
    responses(
        (status = 200, body = AuthResponse)
    ),
)]
#[get("/callback")]
async fn discord_callback(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<OAuthCallbackQuery>,
    data: web::Data<Arc<AuthAppState>>,
) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;
    let state = query.state.clone();

    let db2 = db.clone();

    let request_data = web::block(move || {
        let conn = &mut db.connection()?;
        let data = OAuthRequestData::find(conn, state.as_str())?;
        OAuthRequestData::delete(conn, state.as_str())?;
        Ok::<OAuthRequestData, ApiError>(data)
    })
    .await??;

    let http_client = reqwest::Client::new();

    let token_response = client
        .exchange_code(query.code.clone())
        .set_pkce_verifier(PkceCodeVerifier::new(request_data.pkce_verifier))
        .request_async(&http_client)
        .await
        .map_err(|_| ApiError::new(401, "Failed to request token!"))?;
    let access_token = token_response.access_token();

    let discord_base =
        std::env::var("DISCORD_BASE_URL").unwrap_or_else(|_| "https://discord.com".to_string());
    let discord_user_data = reqwest::Client::new()
        .get(format!("{}/api/users/@me", discord_base))
        .bearer_auth(access_token.secret())
        .send()
        .await
        .map_err(|_| ApiError::new(500, "Failed to request discord data"))?
        .json::<DiscordUser>()
        .await
        .map_err(|_| ApiError::new(500, "Failed to load discord data"))?;

    let (user, roles, scopes) = web::block(move || {
        let conn = &mut db2.connection()?;
        let user = User::upsert(conn, UserUpsert::from(discord_user_data))?;

        let roles = user_roles::table
            .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(user.id))
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let user_privilege_level: i32 = roles
            .iter()
            .map(|role| role.privilege_level)
            .max()
            .unwrap_or(0);

        let all_permissions = permissions::table
            .select((permissions::permission, permissions::privilege_level))
            .load::<(String, i32)>(conn)?;

        let scopes = all_permissions
            .into_iter()
            .filter_map(|(permission, privilege_level)| {
                if user_privilege_level >= privilege_level {
                    Some(permission)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();

        Ok::<_, ApiError>((user, roles, scopes))
    })
    .await??;

    if let Some(callback) = request_data.callback {
        let (token, _expires) = token::create_token(
            UserClaims {
                user_id: user.id,
                is_api_key: false,
            },
            &data.jwt_encoding_key,
            Duration::minutes(5),
            "initial",
        )?;

        let redirect_url = format!("{}?token={}", callback, token);
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, redirect_url))
            .finish());
    }

    let (access_token, access_expires) = token::create_token(
        UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::minutes(30),
        "access",
    )?;

    let (refresh_token, refresh_expires) = token::create_token(
        UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::weeks(2),
        "refresh",
    )?;

    Ok(HttpResponse::Ok().json(AuthResponse {
        access_token,
        access_expires,
        refresh_token,
        refresh_expires,
        user,
        roles,
        scopes,
    }))
}

#[utoipa::path(
    get,
    summary = "[Auth]Refresh Discord auth",
    description = "Get a new access token for Discord auth. If the refresh token is about to expire, will also return a new one.",
    tag = "Authentication",
    responses(
        (status = 200, body = AuthRefreshResponse)
    ),
    security(
        ("refresh_token" = []),
    )
)]
#[get("/refresh")]
async fn discord_refresh(
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

pub(crate) async fn create_discord_client() -> Result<DiscordClient, Box<dyn std::error::Error>> {
    let discord_client_id = ClientId::new(get_secret("DISCORD_CLIENT_ID"));

    let discord_client_secret = ClientSecret::new(get_secret("DISCORD_CLIENT_SECRET"));

    let discord_redirect_uri = RedirectUrl::new(get_secret("DISCORD_REDIRECT_URI"))?;

    let base_discord_url =
        std::env::var("DISCORD_BASE_URL").unwrap_or_else(|_| "https://discord.com".to_string());

    let issuer = IssuerUrl::new(base_discord_url.clone())?;
    let auth_url = AuthUrl::new(format!("{}/oauth2/authorize", base_discord_url).to_string())?;
    let token_url = TokenUrl::new(format!("{}/api/oauth2/token", base_discord_url).to_string())?;

    return Ok(
        CoreClient::new(discord_client_id, issuer, CoreJsonWebKeySet::default())
            .set_client_secret(discord_client_secret)
            .set_auth_uri(auth_url)
            .set_token_uri(token_url)
            .set_redirect_uri(discord_redirect_uri),
    );
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(AuthResponse, AuthRefreshResponse)),
    paths(discord_auth, discord_callback, discord_refresh,)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth/discord")
            .service(discord_auth)
            .service(discord_callback)
            .service(discord_refresh),
    );
}
