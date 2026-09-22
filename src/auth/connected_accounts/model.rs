use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    app_data::db::DbConnection,
    auth::{oauth::OAuthProvider, Authenticated, Permission},
    error_handler::ApiError,
    schema::oauth_connected_accounts,
};

use diesel::prelude::*;
#[derive(Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = oauth_connected_accounts)]
pub struct OAuthConnectedAccount {
    /// The internal UUID of this connected account entry.
    pub id: Uuid,
    /// Internal UUID of the user.
    pub user_id: Uuid,
    /// Which OAuth provider was connected
    pub provider: OAuthProvider,
    /// The user ID of the connected account on the provider's platform
    pub provider_user_id: String,
    /// The username of the connected account on the provider's platform, if available
    pub provider_user_name: Option<String>,
    /// The timestamp when this connected account was added
    pub created_at: DateTime<Utc>,
}

impl OAuthConnectedAccount {
    pub fn find_all_by_user_id(
        conn: &mut DbConnection,
        user_id: Uuid,
        authenticated: &Authenticated,
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
