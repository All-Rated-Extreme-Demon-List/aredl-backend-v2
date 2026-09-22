use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;

#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
/// JSON text message sent after the notifications WebSocket handshake succeeds.
pub struct WebsocketNotification {
    /// Event name: SUBMISSION_CREATED, SUBMISSION_ACCEPTED, SUBMISSION_DENIED,
    /// SUBMISSION_UNDER_CONSIDERATION, SUBMISSION_UNDER_REVIEW, SHIFT_COMPLETED,
    /// SHIFTS_CREATED, or SHIFTS_MISSED.
    pub notification_type: String,
    /// Event payload: a Submission or PlatformerSubmission for submission events, etc..
    /// See the WebSocket endpoint description for more details
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
