use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct WebsocketNotification {
    pub notification_type: String,
    pub data: serde_json::Value,
}

impl WebsocketNotification {
    pub fn send<T: Serialize>(
        notify_tx: &broadcast::Sender<WebsocketNotification>,
        notification_type: impl Into<String>,
        data: &T,
    ) {
        let notification_type = notification_type.into();
        let data = match serde_json::to_value(data) {
            Ok(data) => data,
            Err(error) => {
                tracing::error!(
                    "Failed to serialize {notification_type} websocket notification: {error}"
                );
                return;
            }
        };

        match notify_tx.send(WebsocketNotification {
            notification_type: notification_type.clone(),
            data,
        }) {
            Ok(_) => {}
            Err(error) => {
                tracing::error!(
                    "Failed to send {notification_type} websocket notification: {error}"
                );
            }
        }
    }
}
