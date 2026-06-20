use std::time::Duration;

use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::StreamExt as _;
use tokio::{sync::broadcast, time::interval};
use utoipa::OpenApi;

use crate::{
    auth::{Permission, UserAuth},
    notifications::WebsocketNotification,
};

#[utoipa::path(
    get,
    summary = "[Staff]Subscribe to notifications",
    description = "Upgrades the HTTP connection to a WebSocket. This websocker will receive notifications about events such as accepted/denied submissions. This is mainly used by the discord bot.",
    tag = "Notifications",
    responses(
        (status = 101, description = "Switching Protocols to WebSocket"),
        (status = 401, description = "Unauthorized / invalid or missing token"),
        (status = 403, description = "Forbidden / insufficient permissions"),
    ),
    security(
        ("access_token" = ["NotificationsSubscribe"]),
        ("api_key" = ["NotificationsSubscribe"]),
    ),
)]
#[get(
    "/websocket",
    wrap = "UserAuth::require(Permission::NotificationsSubscribe)"
)]
async fn notifications_websocket(
    req: HttpRequest,
    stream: web::Payload,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, Error> {
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
    components(schemas(WebsocketNotification)),
    paths(notifications_websocket)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/notifications").service(notifications_websocket));
}
