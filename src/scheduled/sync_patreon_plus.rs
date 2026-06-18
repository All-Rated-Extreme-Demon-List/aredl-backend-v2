use crate::app_data::db::{DbAppState, DbConnection};
use crate::aredl::submissions::SubmissionStatus as AredlSubmissionStatus;
use crate::arepl::submissions::SubmissionStatus as AreplSubmissionStatus;
use crate::error_handler::ApiError;
use crate::external_connections::OAuthProvider;
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

#[cfg(test)]
mod tests {
    use super::apply_patreon_plus_sync;
    use crate::aredl::{
        levels::test_utils::create_test_level as create_aredl_test_level,
        submissions::{
            test_utils::create_test_submission as create_aredl_test_submission,
            SubmissionStatus as AredlSubmissionStatus,
        },
    };
    use crate::arepl::{
        levels::test_utils::create_test_level as create_arepl_test_level,
        submissions::{
            test_utils::create_test_submission as create_arepl_test_submission,
            SubmissionStatus as AreplSubmissionStatus,
        },
    };
    use crate::external_connections::OAuthProvider;
    use crate::roles::test_utils::{add_user_to_role, create_test_role_with_desc};
    use crate::schema::{aredl, arepl, oauth_connected_accounts, user_roles};
    use crate::test_utils::init_test_app;
    use crate::users::test_utils::create_test_user;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
    use std::collections::HashSet;

    async fn create_patreon_link(
        db: &std::sync::Arc<crate::app_data::db::DbAppState>,
        user_id: uuid::Uuid,
        patreon_id: &str,
    ) {
        diesel::insert_into(oauth_connected_accounts::table)
            .values((
                oauth_connected_accounts::user_id.eq(user_id),
                oauth_connected_accounts::provider.eq(OAuthProvider::Patreon),
                oauth_connected_accounts::provider_user_id.eq(patreon_id),
                oauth_connected_accounts::provider_user_name.eq::<Option<String>>(None),
            ))
            .execute(&mut db.connection().unwrap())
            .unwrap();
    }

    fn users_with_role(
        db: &std::sync::Arc<crate::app_data::db::DbAppState>,
        role_id: i32,
    ) -> HashSet<uuid::Uuid> {
        user_roles::table
            .filter(user_roles::role_id.eq(role_id))
            .select(user_roles::user_id)
            .load::<uuid::Uuid>(&mut db.connection().unwrap())
            .unwrap()
            .into_iter()
            .collect()
    }

    #[actix_web::test]
    async fn patreon_sync_sets_plus_role_to_active_linked_patrons() {
        let (_, db, _, _) = init_test_app().await;
        let plus_role = create_test_role_with_desc(&db, 5, "plus").await;
        let stale_user_role = create_test_role_with_desc(&db, 1, "stale").await;

        let (active_user, _) = create_test_user(&db, None).await;
        let (inactive_user, _) = create_test_user(&db, None).await;
        let (unlinked_user, _) = create_test_user(&db, None).await;

        create_patreon_link(&db, active_user, "patron_active").await;
        create_patreon_link(&db, inactive_user, "patron_inactive").await;
        add_user_to_role(&db, plus_role, inactive_user).await;
        add_user_to_role(&db, stale_user_role, unlinked_user).await;

        let active_ids =
            HashSet::from(["patron_active".to_string(), "patron_unlinked".to_string()]);

        let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();
        assert_eq!(synced.matched_user_ids, vec![active_user]);
        assert_eq!(synced.removed_user_count, 1);
        assert_eq!(synced.aredl_prioritized_count, 0);
        assert_eq!(synced.arepl_prioritized_count, 0);

        let plus_users = users_with_role(&db, plus_role);
        assert_eq!(plus_users, HashSet::from([active_user]));

        let stale_users = users_with_role(&db, stale_user_role);
        assert_eq!(stale_users, HashSet::from([unlinked_user]));
    }

    #[actix_web::test]
    async fn patreon_sync_clears_plus_role_when_no_active_patrons_match() {
        let (_, db, _, _) = init_test_app().await;
        let plus_role = create_test_role_with_desc(&db, 5, "plus").await;
        let (inactive_user, _) = create_test_user(&db, None).await;

        create_patreon_link(&db, inactive_user, "patron_inactive").await;
        add_user_to_role(&db, plus_role, inactive_user).await;

        let active_ids = HashSet::new();
        let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();
        assert!(synced.matched_user_ids.is_empty());
        assert_eq!(synced.removed_user_count, 1);
        assert!(users_with_role(&db, plus_role).is_empty());
    }

    #[actix_web::test]
    async fn patreon_sync_prioritizes_pending_submissions_from_active_linked_patrons() {
        let (_, db, _, _) = init_test_app().await;
        create_test_role_with_desc(&db, 5, "plus").await;

        let (active_user, _) = create_test_user(&db, None).await;
        let (inactive_user, _) = create_test_user(&db, None).await;

        create_patreon_link(&db, active_user, "patron_active").await;
        create_patreon_link(&db, inactive_user, "patron_inactive").await;

        let aredl_active_level = create_aredl_test_level(&db).await;
        let aredl_active_submission =
            create_aredl_test_submission(aredl_active_level, active_user, &db).await;
        let aredl_inactive_level = create_aredl_test_level(&db).await;
        let aredl_inactive_submission =
            create_aredl_test_submission(aredl_inactive_level, inactive_user, &db).await;
        let aredl_non_pending_level = create_aredl_test_level(&db).await;
        let aredl_non_pending_submission =
            create_aredl_test_submission(aredl_non_pending_level, active_user, &db).await;
        diesel::update(
            aredl::submissions::table
                .filter(aredl::submissions::id.eq(aredl_non_pending_submission)),
        )
        .set(aredl::submissions::status.eq(AredlSubmissionStatus::Claimed))
        .execute(&mut db.connection().unwrap())
        .unwrap();

        let arepl_active_level = create_arepl_test_level(&db).await;
        let arepl_active_submission =
            create_arepl_test_submission(arepl_active_level, active_user, &db).await;
        let arepl_inactive_level = create_arepl_test_level(&db).await;
        let arepl_inactive_submission =
            create_arepl_test_submission(arepl_inactive_level, inactive_user, &db).await;
        let arepl_non_pending_level = create_arepl_test_level(&db).await;
        let arepl_non_pending_submission =
            create_arepl_test_submission(arepl_non_pending_level, active_user, &db).await;
        diesel::update(
            arepl::submissions::table
                .filter(arepl::submissions::id.eq(arepl_non_pending_submission)),
        )
        .set(arepl::submissions::status.eq(AreplSubmissionStatus::Claimed))
        .execute(&mut db.connection().unwrap())
        .unwrap();

        let active_ids = HashSet::from(["patron_active".to_string()]);
        let synced = apply_patreon_plus_sync(&mut db.connection().unwrap(), &active_ids).unwrap();

        assert_eq!(synced.matched_user_ids, vec![active_user]);
        assert_eq!(synced.aredl_prioritized_count, 1);
        assert_eq!(synced.arepl_prioritized_count, 1);

        let aredl_priorities = aredl::submissions::table
            .filter(aredl::submissions::id.eq_any([
                aredl_active_submission,
                aredl_inactive_submission,
                aredl_non_pending_submission,
            ]))
            .select((aredl::submissions::id, aredl::submissions::priority))
            .load::<(uuid::Uuid, bool)>(&mut db.connection().unwrap())
            .unwrap()
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(aredl_priorities[&aredl_active_submission], true);
        assert_eq!(aredl_priorities[&aredl_inactive_submission], false);
        assert_eq!(aredl_priorities[&aredl_non_pending_submission], false);

        let arepl_priorities = arepl::submissions::table
            .filter(arepl::submissions::id.eq_any([
                arepl_active_submission,
                arepl_inactive_submission,
                arepl_non_pending_submission,
            ]))
            .select((arepl::submissions::id, arepl::submissions::priority))
            .load::<(uuid::Uuid, bool)>(&mut db.connection().unwrap())
            .unwrap()
            .into_iter()
            .collect::<std::collections::HashMap<_, _>>();

        assert_eq!(arepl_priorities[&arepl_active_submission], true);
        assert_eq!(arepl_priorities[&arepl_inactive_submission], false);
        assert_eq!(arepl_priorities[&arepl_non_pending_submission], false);
    }
}
