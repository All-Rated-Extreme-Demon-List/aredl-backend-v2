use std::env;
use std::sync::Arc;

use actix_web::{get, HttpResponse, web};
use actix_web::web::redirect;
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::prelude::*;
use openidconnect::{AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreIdTokenVerifier, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use serde::{Deserialize, Serialize};

use crate::auth::app_state::AuthAppState;
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::oauth_requests;

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
    pub nonce: String
}

impl OAuthRequestData {
    fn find(csrf_state: &str) -> Result<Self, ApiError> {
        let request_data = oauth_requests::table
            .filter(oauth_requests::csrf_state.eq(csrf_state))
            .select(OAuthRequestData::as_select())
            .first::<OAuthRequestData>(&mut db::connection()?)?;
        Ok(request_data)
    }

    fn delete(csrf_state: &str) -> Result<(), ApiError> {
        diesel::delete(oauth_requests::table)
            .filter(oauth_requests::csrf_state.eq(csrf_state))
            .execute(&mut db::connection()?)?;
        Ok(())
    }

    fn create(data: &Self) -> Result<(), ApiError> {
        diesel::insert_into(oauth_requests::table)
            .values(data)
            .execute(&mut db::connection()?)?;
        Ok(())
    }
}

#[get("/auth/discord")]
async fn discord_auth(data: web::Data<Arc<AuthAppState>>) -> Result<HttpResponse, ApiError> {
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

    let request_data: OAuthRequestData = OAuthRequestData {
        csrf_state: csrf_state.secret().clone(),
        pkce_verifier: pkce_verifier.secret().to_string(),
        nonce: nonce.secret().to_string(),
    };

    web::block(move || OAuthRequestData::create(&request_data)).await??;

    Ok(HttpResponse::Found().append_header(("Location", authorize_url.to_string())).finish())
}

#[get("/auth/discord/callback")]
async fn discord_callback(query: web::Query<OAuthCallbackQuery>, data: web::Data<Arc<AuthAppState>>) -> Result<HttpResponse, ApiError> {
    let client = &data.discord_client;

    let csrf_state = query.state.clone();
    let request_data = web::block(move || OAuthRequestData::find(&csrf_state)).await??;

    let token_response = client
        .exchange_code(query.code.clone())
        .set_pkce_verifier(PkceCodeVerifier::new(request_data.pkce_verifier))
        .request_async(async_http_client).await
        .map_err(|_| ApiError::new(401, "Failed to request token!".to_string()))?;

    let nonce: Nonce = Nonce::new(request_data.nonce);
    let id_token_verifier: CoreIdTokenVerifier = client.id_token_verifier();
    let test = match token_response.extra_fields().id_token() {
        Some(id_token) => id_token.claims(&id_token_verifier, &nonce).map_err(|_| ApiError::new(401, "Failed to verify ID token!".to_string())),
        None => Err(ApiError::new(500, "No id token provided!".to_string())),
    }?;

    let csrf_state = query.state.clone();
    web::block(move || OAuthRequestData::delete(&csrf_state)).await??;

    Ok(HttpResponse::Ok().json(test))
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
    config
        .service(discord_auth)
        .service(discord_callback);
}