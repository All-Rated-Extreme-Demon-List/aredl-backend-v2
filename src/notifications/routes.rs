use actix_web::{get, web, Error, HttpRequest, HttpResponse};
use actix_ws::{handle, Message};
use futures_util::StreamExt;
use tokio::sync::broadcast;
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

    actix_rt::spawn(async move {
        loop {
            tokio::select! {
                Some(Ok(msg)) = msg_stream.next() => {
                    match msg {
                        Message::Ping(p)    => { let _ = session.pong(&p).await; }
                        Message::Pong(_)    => {  }
                        Message::Close(c)   => { let _ = session.close(c).await; break; }
                        _                   => {  }
                    }
                }
                Ok(note) = rx.recv() => {
                    if let Ok(text) = serde_json::to_string(&note) {
                        let _ = session.text(text).await;
                    }
                }
                else => break,
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
