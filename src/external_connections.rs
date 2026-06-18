use chrono::{DateTime, Utc};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, DbEnum, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::OauthProvider"]
#[DbValueStyle = "PascalCase"]
pub enum OAuthProvider {
    Discord,
    Patreon,
    Google,
    Twitch,
}

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

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::oauth_tokens)]
pub struct OAuthToken {
    pub provider: OAuthProvider,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
