use std::env;
use std::sync::Arc;

use actix_web::{get, HttpResponse, HttpRequest, web};
use chrono::{DateTime, Duration, Utc, TimeZone};
use diesel::prelude::*;
use openidconnect::{AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenVerifier, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use actix_web::http::header;

use crate::auth::app_state::AuthAppState;
use crate::auth::token;
use crate::auth::token::UserClaims;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::{oauth_requests, roles, user_roles, permissions};
use crate::users::{User, UserUpsert};

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
    pub nonce: String,
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

#[derive(Serialize, Deserialize)]
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
        }
    }
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub privilege_level: i32,
    pub role_desc: String,
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    pub access_token: String,
    pub access_expires: DateTime<Utc>,
    pub refresh_token: String,
    pub refresh_expires: DateTime<Utc>,
    pub user: User,
    pub scopes: Vec<String>,
    pub roles: Vec<Role>
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthOptions {
	pub callback: Option<String>,
}

#[get("")]
async fn discord_auth(data: web::Data<Arc<AuthAppState>>, db: web::Data<Arc<DbAppState>>, options: web::Query<AuthOptions>) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;
    let mut conn = db.connection()?;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state, nonce) = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    )
        .add_scope(Scope::new("identify".to_string()))
        .set_pkce_challenge(pkce_challenge.clone())
        .url();

    OAuthRequestData::create(&mut conn, OAuthRequestData {
        csrf_state: csrf_state.secret().clone(),
        pkce_verifier: pkce_verifier.secret().to_string(),
        nonce: nonce.secret().to_string(),
		callback: options.callback.clone(),
    })?;

    Ok(HttpResponse::Found().append_header((header::LOCATION, authorize_url.to_string())).finish())
}

#[get("/callback")]
async fn discord_callback(db: web::Data<Arc<DbAppState>>, query: web::Query<OAuthCallbackQuery>, data: web::Data<Arc<AuthAppState>>) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;

    let mut conn = db.connection()?;
    let state = query.state.clone();
    let request_data = web::block(move || {
        let data = OAuthRequestData::find(&mut conn, state.as_str())?;
        OAuthRequestData::delete(&mut conn, state.as_str())?;
        Ok::<OAuthRequestData, ApiError>(data)
    }).await??;

    let token_response = client
        .exchange_code(query.code.clone())
        .set_pkce_verifier(PkceCodeVerifier::new(request_data.pkce_verifier))
        .request_async(async_http_client).await
        .map_err(|_| ApiError::new(401, "Failed to request token!"))?;

    let nonce: Nonce = Nonce::new(request_data.nonce);
    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    match token_response.extra_fields().id_token() {
        Some(id_token) => id_token.claims(&id_token_verifier, &nonce).map_err(|_| ApiError::new(401, "Failed to verify ID token!")),
        None => Err(ApiError::new(500, "No id token provided!")),
    }?;

    let access_token = token_response.access_token();

    let discord_user_data = reqwest::Client::new()
        .get("https://discord.com/api/users/@me")
        .bearer_auth(access_token.secret())
        .send().await
        .map_err(|_| ApiError::new(500, "Failed to request discord data"))?
        .json::<DiscordUser>().await
        .map_err(|_| ApiError::new(500, "Failed to load discord data"))?;

    let mut conn = db.connection()?;

    let user = web::block(|| User::upsert(db, UserUpsert::from(discord_user_data))).await??;

    let (token, expires) = token::create_token(
        UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::minutes(5),
        "initial",
    )?;

    if let Some(callback) = request_data.callback {
        let redirect_url = format!("{}?token={}", callback, token);
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, redirect_url))
            .finish());
    }

    let (refresh_token, refresh_expires) = token::create_token(
        UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::weeks(2),
        "refresh",
    )?;

    let roles = user_roles::table
        .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
        .filter(user_roles::user_id.eq(user.id))
        .select(Role::as_select())
        .load::<Role>(&mut conn)?;

    let user_privilege_level: i32 = roles
        .iter()
        .map(|role| role.privilege_level)
        .max()
        .unwrap_or(0);

    let all_permissions = permissions::table
        .select((permissions::permission, permissions::privilege_level))
        .load::<(String, i32)>(&mut conn)?;

    let scopes = all_permissions
        .into_iter()
        .filter_map(|(permission, privilege_level)| {
            if user_privilege_level >= privilege_level { Some(permission) } else { None }
        })
        .collect::<Vec<String>>();

    let auth_response = AuthResponse { access_token: token, access_expires: expires, refresh_token, refresh_expires, user, roles, scopes };
	
	Ok(HttpResponse::Ok().json(auth_response))
}

#[get("/refresh")]
async fn discord_refresh(db: web::Data<Arc<DbAppState>>, data: web::Data<Arc<AuthAppState>>, req: HttpRequest) -> Result<HttpResponse, ApiError> {
    let refresh_token = req
        .headers()
        .get(openidconnect::http::header::AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .map(|h| h.strip_prefix("Bearer ").unwrap_or("").to_string());

    if refresh_token.is_none() {
        return Err(ApiError::new(400, "No token provided"));
    }

    let decoded_token_claims = token::decode_token(
        refresh_token.unwrap(),
        &data.jwt_decoding_key,
        &["refresh", "initial"]
    )?;

    let decoded_user_claims = token::decode_user_claims(&decoded_token_claims)?;
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
    let refresh_exp = Utc.timestamp_opt(decoded_token_claims.exp as i64, 0).single().ok_or_else(|| {
        ApiError::new(500, "Failed to parse expiration timestamp")
    })?;

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

pub(crate) async fn create_discord_client() -> Result<CoreClient, Box<dyn std::error::Error>> {
    let discord_client_id = ClientId::new(
        env::var("DISCORD_CLIENT_ID").expect("Missing the DISCORD_CLIENT_ID environment variable."),
    );

    let discord_client_secret = ClientSecret::new(
        env::var("DISCORD_CLIENT_SECRET").expect("Missing the DISCORD_CLIENT_SECRET environment variable."),
    );

    let discord_redirect_uri = RedirectUrl::new(
        env::var("DISCORD_REDIRECT_URI").expect("Missing the DISCORD_REDIRECT_URI environment variable.")
    )?;

    let issuer_url = IssuerUrl::new("https://discord.com".to_string())?;

    let provider_metadata = CoreProviderMetadata::discover_async(issuer_url, async_http_client).await?;
    Ok(CoreClient::from_provider_metadata(
        provider_metadata,
        discord_client_id,
        Some(discord_client_secret),
    ).set_redirect_uri(discord_redirect_uri))
}

pub fn init_discord_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth/discord")
            .service(discord_auth)
            .service(discord_callback)
            .service(discord_refresh)
    );
}