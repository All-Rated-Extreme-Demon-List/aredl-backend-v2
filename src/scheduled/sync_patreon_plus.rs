use crate::app_data::db::{DbAppState, DbConnection};
use crate::aredl::submissions::SubmissionStatus as AredlSubmissionStatus;
use crate::arepl::submissions::SubmissionStatus as AreplSubmissionStatus;
use crate::auth::oauth::OAuthProvider;
use crate::error_handler::ApiError;
use crate::get_secret;
use crate::providers::{context::backend_oauth::OAuthProviderContext, ProvidersAppState};
use crate::schema::{aredl, arepl, oauth_connected_accounts, roles, user_roles};
use chrono::Utc;
use cron::Schedule;
use diesel::{Connection, ExpressionMethods, QueryDsl, RunQueryDsl};
use serde::Deserialize;
use std::collections::HashSet;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use uuid::Uuid;

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

pub struct PatreonActiveMembers {
    total_member_count: usize,
    active_user_ids: HashSet<String>,
}

#[derive(Debug, PartialEq)]
pub struct PatreonPlusSyncResult {
    pub matched_user_ids: Vec<Uuid>,
    pub removed_user_count: usize,
    pub aredl_prioritized_count: usize,
    pub arepl_prioritized_count: usize,
}

pub async fn start_patreon_plus_sync(db: Arc<DbAppState>, providers: Arc<ProvidersAppState>) {
    let schedule = Schedule::from_str(&get_secret("PATREON_SYNC_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let campaign_id = get_secret("PATREON_CAMPAIGN_ID");

    let Some(patreon_auth) = providers.context.patreon_auth.clone() else {
        tracing::warn!("Patreon sync is enabled, but Patreon auth is not configured");
        return;
    };

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .expect("Failed to build reqwest client");
    let patreon_base = patreon_auth.api_base_uri.clone();

    task::spawn(async move {
        loop {
            tracing::info!("Syncing Patreon AREDL+ users");

            let active_members = fetch_active_patreon_user_ids(
                &client,
                &db,
                &patreon_auth,
                &patreon_base,
                &campaign_id,
            )
            .await;

            match active_members {
                Ok(active_members) => match db.connection() {
                    Ok(mut conn) => {
                        match apply_patreon_plus_sync(&mut conn, &active_members.active_user_ids) {
                            Ok(result) => tracing::info!(
                                patreon_members = active_members.total_member_count,
                                linked_members = result.matched_user_ids.len(),
                                removed_members = result.removed_user_count,
                                aredl_prioritized = result.aredl_prioritized_count,
                                arepl_prioritized = result.arepl_prioritized_count,
                                "Synced Patreon AREDL+ users"
                            ),
                            Err(e) => tracing::error!("Failed to apply Patreon AREDL+ sync: {e}"),
                        }
                    }
                    Err(e) => tracing::error!("DB connection failed: {e}"),
                },
                Err(e) => tracing::error!("Failed to fetch Patreon members: {e}"),
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}

async fn fetch_active_patreon_user_ids(
    client: &reqwest::Client,
    db: &DbAppState,
    patreon_auth: &OAuthProviderContext,
    patreon_base: &str,
    campaign_id: &str,
) -> Result<PatreonActiveMembers, ApiError> {
    let access_token = patreon_auth.get_access_token(db).await?;
    let mut active_user_ids = HashSet::new();
    let mut total_member_count = 0;
    let mut cursor: Option<String> = None;

    loop {
        let url = format!(
            "{}/oauth2/v2/campaigns/{}/members",
            patreon_base, campaign_id
        );
        let mut request = client.get(url).bearer_auth(&access_token).query(&[
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
            .map_err(|e| ApiError::new(502, &format!("Failed to request Patreon members: {e}")))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(ApiError::new(
                502,
                &format!("Failed to request Patreon members ({status}): {body}"),
            ));
        }

        let page = response
            .json::<PatreonMembersResponse>()
            .await
            .map_err(|e| {
                ApiError::new(
                    500,
                    &format!("Failed to parse Patreon members response: {e}"),
                )
            })?;

        total_member_count += page.data.len();

        for member in page.data {
            let active = member
                .attributes
                .and_then(|attributes| attributes.patron_status)
                .as_deref()
                == Some("active_patron");

            if !active {
                continue;
            }

            let user_id = member
                .relationships
                .and_then(|relationships| relationships.user)
                .and_then(|user| user.data)
                .map(|data| data.id)
                .unwrap_or(member.id);

            active_user_ids.insert(user_id);
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

    Ok(PatreonActiveMembers {
        total_member_count,
        active_user_ids,
    })
}

pub fn apply_patreon_plus_sync(
    conn: &mut DbConnection,
    active_patreon_user_ids: &HashSet<String>,
) -> Result<PatreonPlusSyncResult, ApiError> {
    let role_id = roles::table
        .filter(roles::role_desc.eq("plus"))
        .select(roles::id)
        .first::<i32>(conn)?;

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

        let (aredl_prioritized_count, arepl_prioritized_count) = if matched_user_ids.is_empty() {
            (0, 0)
        } else {
            let aredl_count = diesel::update(
                aredl::submissions::table
                    .filter(aredl::submissions::status.eq(AredlSubmissionStatus::Pending))
                    .filter(aredl::submissions::submitted_by.eq_any(&matched_user_ids))
                    .filter(aredl::submissions::priority.eq(false)),
            )
            .set(aredl::submissions::priority.eq(true))
            .execute(conn)?;

            let arepl_count = diesel::update(
                arepl::submissions::table
                    .filter(arepl::submissions::status.eq(AreplSubmissionStatus::Pending))
                    .filter(arepl::submissions::submitted_by.eq_any(&matched_user_ids))
                    .filter(arepl::submissions::priority.eq(false)),
            )
            .set(arepl::submissions::priority.eq(true))
            .execute(conn)?;

            (aredl_count, arepl_count)
        };

        Ok::<_, ApiError>(PatreonPlusSyncResult {
            matched_user_ids,
            removed_user_count,
            aredl_prioritized_count,
            arepl_prioritized_count,
        })
    })
}
