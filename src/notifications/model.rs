use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WebsocketNotification {
    pub notification_type: String,
    pub data: serde_json::Value,
}
