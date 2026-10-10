use crate::app_data::db::{DbAppState, DbConnection};
use crate::auth::oauth::OAuthProvider;
use crate::error_handler::{ApiError, StartupError};
use crate::providers::ProvidersAppState;
use crate::scheduled::{parse_startup_schedule, sleep_until_next};
use crate::schema::{oauth_connected_accounts, user_roles};
use crate::utils::patreon::{patreon_plus_role_id, set_users_submissions_to_priority};
use crate::{create_client, get_optional_secret, get_secret};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::task;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct PatreonPlusSyncResult {
    pub matched_user_ids: Vec<Uuid>,
    pub removed_user_count: usize,
    pub prioritized_count: usize,
}

#[derive(Debug, Deserialize)]
struct PatreonMembersResponse {
    data: Vec<PatreonMember>,
    meta: Option<PatreonMeta>,
}

#[derive(Debug, Deserialize)]
struct PatreonMember {
    id: String,
    attributes: Option<PatreonMemberAttributes>,
    relationships: Option<PatreonMemberRelationships>,
}

#[derive(Debug, Deserialize)]
struct PatreonMemberAttributes {
    patron_status: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PatreonMemberRelationships {
    user: Option<PatreonRelationshipOne>,
}

#[derive(Debug, Deserialize)]
struct PatreonRelationshipOne {
    data: Option<PatreonRelationshipData>,
}

#[derive(Debug, Deserialize)]
struct PatreonRelationshipData {
    id: String,
}

#[derive(Debug, Deserialize)]
struct PatreonMeta {
    pagination: Option<PatreonPagination>,
}

#[derive(Debug, Deserialize)]
struct PatreonPagination {
    cursors: Option<PatreonCursors>,
}

#[derive(Debug, Deserialize)]
struct PatreonCursors {
    next: Option<String>,
}

pub async fn start_patreon_plus_sync(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<(), StartupError> {
    let Some(schedule_config) =
        get_optional_secret("PATREON_SYNC_SCHEDULE").filter(|value| !value.is_empty())
    else {
        tracing::info!("PATREON_SYNC_SCHEDULE not set, patreon sync is disabled");
        return Ok(());
    };
    let schedule = parse_startup_schedule("PATREON_SYNC_SCHEDULE", &schedule_config)?;
    get_secret("PATREON_CAMPAIGN_ID")?;
    if providers.context.patreon_auth.is_none() {
        tracing::warn!("Patreon sync is enabled, but Patreon auth is not configured");
        return Ok(());
    }
    task::spawn(async move {
        loop {
            if let Err(error) = sync(db.clone(), providers.clone()).await {
                tracing::error!("Failed to apply Patreon AREDL+ sync: {error}");
            }
            sleep_until_next(&schedule).await;
        }
    });
    Ok(())
}

pub async fn sync(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<PatreonPlusSyncResult, ApiError> {
    let patreon_auth = providers
        .context
        .patreon_auth
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Patreon auth is not configured"))?;

    let campaign_id = get_secret("PATREON_CAMPAIGN_ID").map_err(ApiError::InternalServerError)?;
    let client = create_client().map_err(ApiError::InternalServerError)?;
    let token = patreon_auth.get_access_token(&db).await?;

    let active_user_ids =
        fetch_active_patreon_user_ids(&client, &token, &patreon_auth.api_base_uri, &campaign_id)
            .await?;
    actix_web::web::block(move || apply_patreon_plus_sync(&mut db.connection()?, &active_user_ids))
        .await?
}

async fn fetch_active_patreon_user_ids(
    client: &reqwest::Client,
    access_token: &str,
    patreon_base: &str,
    campaign_id: &str,
) -> Result<HashSet<String>, ApiError> {
    let mut active_user_ids = HashSet::new();
    let mut cursor: Option<String> = None;

    loop {
        let url = format!("{patreon_base}/oauth2/v2/campaigns/{campaign_id}/members");
        let mut request = client.get(url).bearer_auth(access_token).query(&[
            ("include", "user"),
            ("fields[member]", "patron_status"),
            ("fields[user]", "full_name,vanity"),
            ("page[count]", "1000"),
        ]);

        if let Some(cursor) = &cursor {
            request = request.query(&[("page[cursor]", cursor)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Failed to request Patreon members: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::BadGateway(format!(
                "Failed to request Patreon members ({status}): {body}"
            )));
        }

        let page = response
            .json::<PatreonMembersResponse>()
            .await
            .map_err(|e| {
                ApiError::BadGateway(format!("Failed to parse Patreon members response: {e}"))
            })?;

        for member in page.data {
            if !member.is_active() {
                continue;
            }

            active_user_ids.insert(member.user_id().unwrap_or(member.id));
        }

        cursor = page
            .meta
            .and_then(|meta| meta.pagination)
            .and_then(|pagination| pagination.cursors)
            .and_then(|cursors| cursors.next);

        if cursor.is_none() {
            break;
        }
    }

    Ok(active_user_ids)
}

pub fn apply_patreon_plus_sync(
    conn: &mut DbConnection,
    active_patreon_user_ids: &HashSet<String>,
) -> Result<PatreonPlusSyncResult, ApiError> {
    let role_id = patreon_plus_role_id(conn)?;

    let matched_user_ids = if active_patreon_user_ids.is_empty() {
        Vec::new()
    } else {
        oauth_connected_accounts::table
            .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
            .filter(oauth_connected_accounts::provider_user_id.eq_any(active_patreon_user_ids))
            .select(oauth_connected_accounts::user_id)
            .load::<Uuid>(conn)?
    };

    conn.transaction(|conn| {
        let existing_plus_user_ids = user_roles::table
            .filter(user_roles::role_id.eq(role_id))
            .select(user_roles::user_id)
            .load::<Uuid>(conn)?;

        let matched_user_set = matched_user_ids.iter().copied().collect::<HashSet<_>>();
        let removed_user_count = existing_plus_user_ids
            .iter()
            .filter(|user_id| !matched_user_set.contains(user_id))
            .count();

        diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id))).execute(conn)?;

        if !matched_user_ids.is_empty() {
            diesel::insert_into(user_roles::table)
                .values(
                    matched_user_ids
                        .iter()
                        .map(|user_id| {
                            (
                                user_roles::role_id.eq(role_id),
                                user_roles::user_id.eq(*user_id),
                            )
                        })
                        .collect::<Vec<_>>(),
                )
                .execute(conn)?;
        }

        let prioritized_count = set_users_submissions_to_priority(conn, &matched_user_ids)?;

        Ok::<_, ApiError>(PatreonPlusSyncResult {
            matched_user_ids,
            removed_user_count,
            prioritized_count,
        })
    })
}

impl PatreonMember {
    fn is_active(&self) -> bool {
        self.attributes
            .as_ref()
            .and_then(|attributes| attributes.patron_status.as_deref())
            == Some("active_patron")
    }

    fn user_id(&self) -> Option<String> {
        self.relationships
            .as_ref()
            .and_then(|relationships| relationships.user.as_ref())
            .and_then(|user| user.data.as_ref())
            .map(|data| data.id.clone())
    }
}
