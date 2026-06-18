use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    auth::{oauth::OAuthProvider, Authenticated, Permission},
    error_handler::ApiError,
    schema::oauth_connected_accounts,
};

#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::oauth_connected_accounts)]
pub struct OAuthConnectedAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub provider: OAuthProvider,
    pub provider_user_id: String,
    pub provider_user_name: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl OAuthConnectedAccount {
    pub fn find_all_by_user_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        authenticated: Authenticated,
    ) -> Result<Vec<Self>, ApiError> {
        if authenticated.user_id != user_id
            && !authenticated.has_permission(conn, Permission::ExternalConnectionsManage)?
        {
            return Ok(Vec::new());
        }

        Ok(oauth_connected_accounts::table
            .filter(oauth_connected_accounts::user_id.eq(user_id))
            .select(OAuthConnectedAccount::as_select())
            .load::<Self>(conn)?)
    }
}
