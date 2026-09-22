use std::time::Duration;

use actix_web::{get, web, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::StreamExt as _;
use tokio::{sync::broadcast, time::interval};
use utoipa::OpenApi;

use crate::{
    auth::{Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    notifications::WebsocketNotification,
    shifts::ShiftInsert,
};

#[utoipa::path(
    get,
    summary = "[Staff]Subscribe to notifications",
    description = "Upgrades the HTTP connection to a WebSocket for staff notifications, mainly used by the Discord bot.

Send a WebSocket handshake with `Upgrade: websocket`, `Connection: Upgrade`, `Sec-WebSocket-Version: 13`, and `Sec-WebSocket-Key`, along with the bearer token.

After the `101` upgrade, notifications are sent as JSON text messages using `WebsocketNotification`.

| notification_type | data payload | When sent |
|---|---|---|
| `SUBMISSION_CREATED` | `Submission` or `PlatformerSubmission` | After a classic or platformer submission is created. |
| `SUBMISSION_ACCEPTED` | `Submission` or `PlatformerSubmission` | After a reviewer changes the submission status to Accepted. |
| `SUBMISSION_DENIED` | `Submission` or `PlatformerSubmission` | After a reviewer changes the submission status to Denied. |
| `SUBMISSION_UNDER_CONSIDERATION` | `Submission` or `PlatformerSubmission` | After a reviewer changes the submission status to UnderConsideration. |
| `SUBMISSION_UNDER_REVIEW` | `Submission` or `PlatformerSubmission` | After a reviewer changes the submission status to UnderReview. |
| `SHIFT_COMPLETED` | `Shift` | When a reviewer completes a shift after reviewing a submission. |
| `SHIFTS_CREATED` | Array of `ShiftInsert` | Emitted whenever the scheduled job to create shifts based on recurrent-shifts runs. |
| `SHIFTS_MISSED` | Array of `Shift` | Emitted when the cleanup job expires running shifts that haven't been completed in time. |

The server sends Ping frames every 30 seconds and responds to client Ping frames with Pong.
",
    tag = "Notifications",
    responses(
        (status = 101, description = "Switching Protocols to WebSocket. Following notifications are WebSocket text messages.",
            headers(
                ("Upgrade" = String),
                ("Connection" = String),
                ("Sec-WebSocket-Accept" = String)
            )
        ),
        (status = 400, description = "Invalid WebSocket handshake: missing or invalid Upgrade/Connection headers, missing or unsupported Sec-WebSocket-Version, or missing Sec-WebSocket-Key.", body = ErrorResponse),
    ),
    security(("bearer_token" = ["NotificationsSubscribe"])),
)]
#[get(
    "/websocket",
    wrap = "UserAuth::require(Permission::NotificationsSubscribe)"
)]
async fn notifications_websocket(
    req: HttpRequest,
    stream: web::Payload,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    let (res, mut session, mut msg_stream) = handle(&req, stream)?;
    let mut rx = notify_tx.subscribe();
    let mut heartbeat = interval(Duration::from_secs(30));

    actix_rt::spawn(async move {
        loop {
            tokio::select! {
                _ = heartbeat.tick() => {
                     if session.ping(&[]).await.is_err() {
                    break;
                }
                }

               message = msg_stream.next() => {
                match message {
                    Some(Ok(Message::Ping(payload))) => {
                        if session.pong(&payload).await.is_err() {
                            break;
                        }
                    }
                    Some(Ok(Message::Close(reason))) => {
                        if let Err(error) = session.close(reason).await {
                            tracing::debug!("Failed to close WebSocket session: {error}");
                        }
                        break;
                    }
                    Some(Ok(Message::Pong(_) | _))  => {}
                    Some(Err(error)) => {
                        tracing::debug!("WebSocket protocol error: {error}");
                        break;
                    }
                    None => break,
                }
            }
            notification = rx.recv() => {
                match notification {
                    Ok(notification) => {
                        let text = match serde_json::to_string(&notification) {
                            Ok(text) => text,
                            Err(error) => {
                                tracing::error!(
                                    "failed to serialize WebSocket notification: {error}"
                                );
                                continue;
                            }
                        };

                        if session.text(text).await.is_err() {
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        tracing::warn!(
                            "WebSocket subscriber skipped {skipped} notifications"
                        );
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
                }
            }
        }
    });
    Ok(res)
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(WebsocketNotification, ShiftInsert)),
    paths(notifications_websocket)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/notifications").service(notifications_websocket));
}
