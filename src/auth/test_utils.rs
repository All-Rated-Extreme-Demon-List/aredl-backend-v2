use crate::app_data::db::DbAppState;
use crate::auth::oauth::OAuthProvider;
use crate::schema::{
    oauth_connected_accounts, oauth_requests,
    users::dsl::{access_valid_after, id as user_id_col, users},
};
use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _};
use std::sync::Arc;
use uuid::Uuid;

pub fn access_valid_after_for_user(db: &Arc<DbAppState>, user_id: Uuid) -> DateTime<Utc> {
    users
        .filter(user_id_col.eq(user_id))
        .select(access_valid_after)
        .first(&mut db.connection().unwrap())
        .unwrap()
}

pub fn patreon_connections_for_user(db: &Arc<DbAppState>, user_id: Uuid) -> Vec<String> {
    oauth_connected_accounts::table
        .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
        .filter(oauth_connected_accounts::user_id.eq(user_id))
        .select(oauth_connected_accounts::provider_user_id)
        .load::<String>(&mut db.connection().unwrap())
        .unwrap()
}

pub fn seed_connected_account(
    db: &Arc<DbAppState>,
    user_id: Uuid,
    provider: OAuthProvider,
    provider_user_id: &str,
    provider_user_name: Option<&str>,
) {
    diesel::insert_into(oauth_connected_accounts::table)
        .values((
            oauth_connected_accounts::user_id.eq(user_id),
            oauth_connected_accounts::provider.eq(provider),
            oauth_connected_accounts::provider_user_id.eq(provider_user_id),
            oauth_connected_accounts::provider_user_name.eq(provider_user_name.map(str::to_owned)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();
}

pub fn seed_oauth_request(
    db: &Arc<DbAppState>,
    provider: OAuthProvider,
    state: &str,
    pkce_verifier: Option<&str>,
    callback: Option<&str>,
    user_id: Option<Uuid>,
) {
    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq(state),
            oauth_requests::pkce_verifier.eq(pkce_verifier),
            oauth_requests::callback.eq(callback),
            oauth_requests::provider.eq(provider),
            oauth_requests::user_id.eq(user_id),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();
}
