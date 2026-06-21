use crate::app_data::db::DbAppState;
use crate::auth::oauth::OAuthProvider;
use crate::auth::oauth::{exchange_oauth_code, OAuthCallbackQuery, OAuthRequestData};
use crate::auth::OAuthOptions;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::ApiError;
use crate::providers::ProvidersAppState;
use crate::schema::oauth_connected_accounts;
use actix_http::header;
use actix_web::{get, web, HttpResponse};
use diesel::{
    BoolExpressionMethods as _, Connection as _, ExpressionMethods as _, QueryDsl as _,
    RunQueryDsl as _,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use utoipa::{OpenApi, ToSchema};

#[derive(Debug, Serialize, ToSchema)]
struct PatreonLinkResponse {
    authorize_url: String,
}

#[derive(Debug, Serialize, ToSchema)]
struct PatreonLinkedResponse {
    provider_user_id: String,
}

#[derive(Debug, Deserialize)]
struct PatreonIdentityResponse {
    data: PatreonIdentityData,
}

#[derive(Debug, Deserialize)]
struct PatreonIdentityData {
    id: String,
    attributes: Option<PatreonIdentityAttributes>,
}

#[derive(Debug, Deserialize)]
struct PatreonIdentityAttributes {
    full_name: Option<String>,
    vanity: Option<String>,
}

#[utoipa::path(
    get,
    summary = "[Auth]Link Patreon account",
    description = "Starts a Patreon OAuth flow to link the authenticated AREDL user to a Patreon account.",
    tag = "Authentication",
    params(
        ("callback" = Option<String>, Query, description = "Optional URL to redirect to after Patreon linking")
    ),
    responses(
        (status = 200, body = PatreonLinkResponse)
    ),
    security(("access_token" = []), ("api_key" = []))
)]
#[get("/link", wrap = "UserAuth::load()")]
async fn patreon_link(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
    authenticated: Authenticated,
    options: web::Query<OAuthOptions>,
) -> Result<HttpResponse, ApiError> {
    if options.callback.is_some() {
        options.validate()?;
    }

    let patreon_auth = providers
        .context
        .patreon_auth
        .clone()
        .ok_or_else(|| ApiError::ServiceUnavailable("Patreon integration is not configured"))?;

    let callback = options.callback.clone();
    let user_id = authenticated.user_id;

    let authorize_url = web::block(move || {
        OAuthRequestData::init_request(
            &mut db.connection()?,
            patreon_auth.user_oauth()?,
            OAuthProvider::Patreon,
            callback,
            Some(user_id),
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(PatreonLinkResponse { authorize_url }))
}

#[utoipa::path(
    get,
    summary = "Patreon Callback",
    description = "Completes the Patreon OAuth flow and links the Patreon account to the user who started the flow.",
    tag = "Authentication",
    responses(
        (status = 200, body = PatreonLinkedResponse),
        (status = 302)
    )
)]
#[get("/callback")]
async fn patreon_callback(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
    query: web::Query<OAuthCallbackQuery>,
) -> Result<HttpResponse, ApiError> {
    let patreon_auth = providers
        .context
        .patreon_auth
        .clone()
        .ok_or_else(|| ApiError::ServiceUnavailable("Patreon integration is not configured"))?;

    let state = query.state.clone();
    let db_for_request = db.clone();
    let request_data = web::block(move || {
        OAuthRequestData::consume_request(
            &mut db_for_request.connection()?,
            OAuthProvider::Patreon,
            &state,
        )
    })
    .await??;

    let user_id = request_data
        .user_id
        .ok_or_else(|| ApiError::BadRequest("Invalid Patreon OAuth request"))?;

    let access_token = exchange_oauth_code(
        &patreon_auth.user_oauth()?.client,
        &query.code,
        request_data.pkce_verifier.clone(),
    )
    .await?;
    let patreon_user = fetch_patreon_identity(&access_token, &patreon_auth.api_base_uri).await?;
    let provider_user_id = patreon_user.data.id;
    let provider_user_name = patreon_user
        .data
        .attributes
        .and_then(|attributes| attributes.full_name.or(attributes.vanity));

    let provider_user_id_for_db = provider_user_id.clone();
    let provider_user_name_for_db = provider_user_name.clone();
    let db_for_link = db.clone();
    web::block(move || {
        let conn = &mut db_for_link.connection()?;
        conn.transaction(|conn| {
            diesel::delete(
                oauth_connected_accounts::table
                    .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
                    .filter(
                        oauth_connected_accounts::provider_user_id
                            .eq(&provider_user_id_for_db)
                            .or(oauth_connected_accounts::user_id.eq(user_id)),
                    ),
            )
            .execute(conn)?;

            diesel::insert_into(oauth_connected_accounts::table)
                .values((
                    oauth_connected_accounts::user_id.eq(user_id),
                    oauth_connected_accounts::provider.eq(OAuthProvider::Patreon),
                    oauth_connected_accounts::provider_user_id.eq(provider_user_id_for_db),
                    oauth_connected_accounts::provider_user_name.eq(provider_user_name_for_db),
                ))
                .execute(conn)?;

            Ok::<_, ApiError>(())
        })
    })
    .await??;

    if let Some(callback) = request_data.callback {
        let mut callback_url = Url::parse(&callback)
            .map_err(|_err| ApiError::InternalServerError("Invalid callback URL"))?;
        callback_url
            .query_pairs_mut()
            .append_pair("patreon", "linked");
        return Ok(HttpResponse::Found()
            .append_header((header::LOCATION, callback_url.to_string()))
            .finish());
    }

    Ok(HttpResponse::Ok().json(PatreonLinkedResponse { provider_user_id }))
}

async fn fetch_patreon_identity(
    access_token: &str,
    patreon_base: &str,
) -> Result<PatreonIdentityResponse, ApiError> {
    let url = format!("{patreon_base}/oauth2/v2/identity");

    let response = reqwest::Client::new()
        .get(url)
        .bearer_auth(access_token)
        .query(&[("fields[user]", "full_name,vanity")])
        .send()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Failed to request patreon identity: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::BadGateway(format!(
            "Failed to request patreon identity ({status}): {body}"
        )));
    }

    response
        .json::<PatreonIdentityResponse>()
        .await
        .map_err(|e| {
            ApiError::BadGateway(format!("Failed to parse patreon identity response: {e}"))
        })
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(PatreonLinkResponse, PatreonLinkedResponse)),
    paths(patreon_link, patreon_callback)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/patreon")
            .service(patreon_link)
            .service(patreon_callback),
    );
}
