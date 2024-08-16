use std::env;
use std::sync::Arc;

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
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::oauth_requests;
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
	pub opener_origin: Option<String>,
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
	pub opener_origin: Option<String>,
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
		opener_origin: options.opener_origin.clone(),
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

    if request_data.opener_origin.is_some() {
		let script_data = serde_json::json!({
			"data": auth_response,
		});
	
		let script = format!(
			"<script>
				if (window.opener) {{
					window.opener.postMessage({}, '{}');
				}}
				window.close();
			</script>",
			script_data,
			request_data.opener_origin.unwrap(),
		);
	
		return Ok(HttpResponse::Ok().content_type("text/html").body(script));
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