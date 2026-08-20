use crate::app_data::db::DbAppState;
use crate::auth::oauth::OAuthProvider;
use crate::auth::oauth::{exchange_oauth_code, OAuthCallbackQuery, OAuthRequestData};
use crate::auth::OAuthOptions;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::ApiError;
use crate::get_secret;
use crate::providers::ProvidersAppState;
use crate::schema::oauth_connected_accounts;
use crate::utils::patreon::grant_patreon_plus;
use actix_http::header;
use actix_web::web::Json;
use actix_web::{get, post, web, HttpResponse};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use url::Url;
use utoipa::{OpenApi, ToSchema};

use diesel::prelude::*;

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
    #[serde(default)]
    included: Vec<PatreonMembership>,
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

#[derive(Debug, Deserialize)]
struct PatreonMembership {
    #[serde(rename = "type")]
    resource_type: String,
    attributes: Option<PatreonMembershipAttributes>,
    relationships: Option<PatreonMembershipRelationships>,
}

#[derive(Debug, Deserialize)]
struct PatreonMembershipAttributes {
    patron_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatreonMembershipRelationships {
    campaign: Option<PatreonRelationshipOne>,
}

#[derive(Debug, Deserialize)]
struct PatreonRelationshipOne {
    data: Option<PatreonRelationshipData>,
}

#[derive(Debug, Deserialize)]
struct PatreonRelationshipData {
    id: String,
}

#[utoipa::path(
    post,
    summary = "[Auth]Link Patreon account",
    description = "Starts a Patreon OAuth flow to link the authenticated AREDL user to a Patreon account.",
    tag = "Authentication",
    request_body = OAuthOptions,
    responses(
        (status = 200, body = PatreonLinkResponse)
    ),
    security(("access_token" = []), ("api_key" = []))
)]
#[post("", wrap = "UserAuth::load()")]
async fn patreon_link(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
    authenticated: Authenticated,
    options: Option<web::Json<OAuthOptions>>,
) -> Result<HttpResponse, ApiError> {
    let options = options.map(Json::into_inner).unwrap_or_default();

    options.validate()?;

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
        (status = 409, description = "Patreon account is already linked to another user"),
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
    let campaign_id = get_secret("PATREON_CAMPAIGN_ID")?;
    let patreon_user = fetch_patreon_identity(
        &providers.context.http,
        &access_token,
        &patreon_auth.api_base_uri,
    )
    .await?;

    if !patreon_user.is_active_member_of(&campaign_id) {
        return Err(ApiError::Forbidden(
            "This Patreon account is not a member of the AREDL Patreon. Make sure you are trying to connect the right Patreon account, and that you are subscribed to the correct AREDL Patreon page.",
        ));
    }
    let provider_user_id = patreon_user.data.id.clone();
    let provider_user_name = patreon_user.provider_user_name();

    let provider_user_id_for_db = provider_user_id.clone();
    let provider_user_name_for_db = provider_user_name.clone();
    let db_for_link = db.clone();
    web::block(move || {
        let conn = &mut db_for_link.connection()?;
        conn.transaction(|conn| {
            let existing_user_id = oauth_connected_accounts::table
                .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
                .filter(oauth_connected_accounts::provider_user_id.eq(&provider_user_id_for_db))
                .select(oauth_connected_accounts::user_id)
                .first::<uuid::Uuid>(conn)
                .optional()?;

            if existing_user_id.is_some_and(|existing_user_id| existing_user_id != user_id) {
                return Err(ApiError::Conflict(
                    "This Patreon account is already linked to another user",
                ));
            }

            diesel::delete(
                oauth_connected_accounts::table
                    .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
                    .filter(oauth_connected_accounts::user_id.eq(user_id)),
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

            grant_patreon_plus(conn, user_id)?;

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
    client: &reqwest::Client,
    access_token: &str,
    patreon_base: &str,
) -> Result<PatreonIdentityResponse, ApiError> {
    let url = format!("{patreon_base}/oauth2/v2/identity");

    let response = client
        .get(url)
        .bearer_auth(access_token)
        .query(&[
            ("include", "memberships,memberships.campaign"),
            ("fields[user]", "full_name,vanity"),
            ("fields[member]", "patron_status"),
        ])
        .send()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Failed to request Patreon identity: {e}")))?;

    let status = response.status();
    if !status.is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(ApiError::BadGateway(format!(
            "Failed to request Patreon identity ({status}): {body}"
        )));
    }

    response
        .json::<PatreonIdentityResponse>()
        .await
        .map_err(|e| {
            ApiError::BadGateway(format!("Failed to parse Patreon identity response: {e}"))
        })
}

impl PatreonIdentityResponse {
    fn is_active_member_of(&self, campaign_id: &str) -> bool {
        self.included
            .iter()
            .any(|membership| membership.is_active_member_of(campaign_id))
    }

    fn provider_user_name(self) -> Option<String> {
        let attributes = self.data.attributes?;
        attributes.full_name.or(attributes.vanity)
    }
}

impl PatreonMembership {
    fn is_active_member_of(&self, campaign_id: &str) -> bool {
        self.resource_type == "member"
            && self
                .attributes
                .as_ref()
                .and_then(|attributes| attributes.patron_status.as_deref())
                == Some("active_patron")
            && self
                .relationships
                .as_ref()
                .and_then(|relationships| relationships.campaign.as_ref())
                .and_then(|campaign| campaign.data.as_ref())
                .is_some_and(|campaign| campaign.id == campaign_id)
    }
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
