use std::sync::Arc;

use actix_http::header;
use actix_web::{get, web, HttpResponse};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;

use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};

use crate::app_data::auth::AuthAppState;
use crate::app_data::db::DbAppState;
use crate::auth::oauth::OAuthProvider;
use crate::auth::oauth::{exchange_oauth_code, OAuthCallbackQuery, OAuthRequestData};
use crate::auth::token::{self, UserClaims};
use crate::auth::OAuthOptions;
use crate::error_handler::ApiError;
use crate::providers::ProvidersAppState;
use crate::roles::Role;
use crate::schema::{permissions, roles, user_roles};
use crate::users::{User, UserUpsert};

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
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
    options: web::Query<OAuthOptions>,
) -> Result<HttpResponse, ApiError> {
    if options.callback.is_some() {
        options.validate()?;
    }

    let discord_provider = providers
        .context
        .discord_auth
        .clone()
        .ok_or_else(|| ApiError::ServiceUnavailable("Discord integration is not configured"))?;

    let authorize_url = web::block(move || {
        OAuthRequestData::init_request(
            &mut db.connection()?,
            discord_provider.user_oauth()?,
            OAuthProvider::Discord,
            options.callback.clone(),
            None,
        )
    })
    .await??;

    Ok(HttpResponse::Found()
        .append_header((header::LOCATION, authorize_url))
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
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let discord_provider = providers
        .context
        .discord_auth
        .clone()
        .ok_or_else(|| ApiError::ServiceUnavailable("Discord integration is not configured"))?;
    let state = query.state.clone();

    let db2 = db.clone();

    let request_data = web::block(move || {
        let conn = &mut db.connection()?;
        OAuthRequestData::consume_request(conn, OAuthProvider::Discord, state.as_str())
    })
    .await??;

    let access_token = exchange_oauth_code(
        &discord_provider.user_oauth()?.client,
        &query.code,
        request_data.pkce_verifier,
    )
    .await?;

    let discord_user_data = reqwest::Client::new()
        .get(format!("{}/api/users/@me", discord_provider.api_base_uri))
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|_err| ApiError::BadGateway("Failed to request discord data"))?
        .json::<DiscordUser>()
        .await
        .map_err(|_err| ApiError::BadGateway("Failed to load discord data"))?;

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
                (user_privilege_level >= privilege_level).then_some(permission)
            })
            .collect::<Vec<String>>();

        Ok::<_, ApiError>((user, roles, scopes))
    })
    .await??;

    if let Some(callback) = request_data.callback {
        let (token, _expires) = token::create_token(
            &UserClaims {
                user_id: user.id,
                is_api_key: false,
            },
            &data.jwt_encoding_key,
            Duration::minutes(5),
            "initial",
        )?;

        let redirect_url = format!("{callback}?token={token}");
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, redirect_url))
            .finish());
    }

    let (access_token, access_expires) = token::create_token(
        &UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::minutes(30),
        "access",
    )?;

    let (refresh_token, refresh_expires) = token::create_token(
        &UserClaims {
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
        scopes,
        roles,
    }))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(AuthResponse)),
    paths(discord_auth, discord_callback,)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/discord")
            .service(discord_auth)
            .service(discord_callback),
    );
}
