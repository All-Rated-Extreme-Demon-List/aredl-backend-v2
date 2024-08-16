use std::env;
use std::sync::Arc;
use actix_session::Session;

use actix_web::{get, HttpResponse, web};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use openidconnect::{AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenVerifier, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use serde::{Deserialize, Serialize};
use actix_web::http::header;

use crate::auth::app_state::AuthAppState;
use crate::auth::token;
use crate::auth::token::UserClaims;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::{User, UserUpsert};

#[derive(Deserialize)]
struct OAuthCallbackQuery {
    code: AuthorizationCode,
    state: String,
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
            global_name: user.global_name.or(Some(user.username)),
            discord_id: Some(user.id),
            placeholder: false,
            country: None,
            discord_avatar: user.avatar,
            discord_banner: user.banner,
            discord_accent_color: user.accent_color,
        }
    }
}

#[derive(Debug, Serialize)]
struct AuthResponse {
    pub token: String,
    pub expires: DateTime<Utc>,
    pub user: User,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthOptions {
    pub use_message: Option<bool>
}

#[get("")]
async fn discord_auth(data: web::Data<Arc<AuthAppState>>, session: Session, options: web::Query<AuthOptions>) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let (authorize_url, csrf_state, nonce) = client.authorize_url(
        CoreAuthenticationFlow::AuthorizationCode,
        CsrfToken::new_random,
        Nonce::new_random,
    )
        .add_scope(Scope::new("identify".to_string()))
        .set_pkce_challenge(pkce_challenge.clone())
        .url();

    session.insert("csrf_state", csrf_state.secret().clone())
        .map_err(|_| ApiError::new(400, "Session error"))?;

    session.insert("pkce_verifier", pkce_verifier.secret().to_string())
        .map_err(|_| ApiError::new(400, "Session error"))?;

    session.insert("nonce", nonce.secret().to_string())
        .map_err(|_| ApiError::new(400, "Session error"))?;

    session.insert("use_message", "true")
        .map_err(|_| ApiError::new(400, "Session error"))?;

    Ok(HttpResponse::Found().append_header((header::LOCATION, authorize_url.to_string())).finish())
}

#[get("/callback")]
async fn discord_callback(db: web::Data<Arc<DbAppState>>, query: web::Query<OAuthCallbackQuery>, session: Session, data: web::Data<Arc<AuthAppState>>) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;

    let session_state = session.remove_as::<String>("csrf_state");

    if match session_state {
        Some(Ok(state)) => query.state != state,
        _ => true
    } {
        return Err(ApiError::new(400, "Session error"))
    }

    let pkce_verifier = session.remove_as::<String>("pkce_verifier");

    let pkce_verifier = match pkce_verifier {
        Some(Ok(pkce)) => Ok(pkce),
        _ => Err(ApiError::new(400, "Session error"))
    }?;

    let nonce = session.remove_as::<String>("nonce");

    let nonce = match nonce {
        Some(Ok(nonce)) => Ok(nonce),
        _ => Err(ApiError::new(400, "Session error"))
    }?;

    let token_response = client
        .exchange_code(query.code.clone())
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier))
        .request_async(async_http_client).await
        .map_err(|_| ApiError::new(401, "Failed to request token!"))?;

    let nonce: Nonce = Nonce::new(nonce);
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

    let user = web::block(|| User::upsert(db, UserUpsert::from(discord_user_data))).await??;

    let (token, expires) = token::create_token(
        UserClaims {
            user_id: user.id,
            is_api_key: false,
        },
        &data.jwt_encoding_key,
        Duration::weeks(52),
    )?;

    let auth_response = AuthResponse { token, expires, user };

    let use_message = session.remove_as::<bool>("use_message");

    let use_message = match use_message {
        Some(Ok(use_message)) => Ok(use_message),
        _ => Err(ApiError::new(400, "Session error"))
    }?;

    if (use_message) {
        let script = format!(
            "<script>
                        window.opener.postMessage({{ data: '{}' }}, window.location.origin);
                        window.close();
                    </script>",
            serde_json::to_string(&auth_response)
                .map_err(|_| ApiError::new(400, "Script generation error"))?
        );
        return Ok(HttpResponse::Ok().content_type("text/html").body(script))
    }
    Ok(HttpResponse::Ok().json(auth_response))
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
    );
}