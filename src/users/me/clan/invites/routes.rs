use std::sync::Arc;
use uuid::Uuid;
use actix_web::{get, post, web, HttpResponse};
use utoipa::OpenApi;
use crate::auth::{UserAuth, Authenticated};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::clans::ClanInvite;
use crate::users::me::clan::invites::ClanInviteResolved;


#[utoipa::path(
    get,
    summary = "[Auth]Get my invites",
    description = "Get the list of clan invites you've received",
    tag = "Users - Me",
    responses(
        (status = 200, body = [ClanInviteResolved])
    ),
	security(
		("access_token" = []),
		("api_key" = []),
	)
)]
#[get("", wrap="UserAuth::load()")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
	authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanInvite::find_all_me_invites(&mut conn, authenticated.user_id)
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Accept clan invite",
    description = "Accepts an invite to join a clan. This will remove all your other invites and add you to the clan.",
    tag = "Users - Me",
    params(
        ("invite_id" = Uuid, description = "The internal UUID of the invite")
    ),
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("/{invite_id}/accept", wrap="UserAuth::load()")]
async fn accept(
    db: web::Data<Arc<DbAppState>>,
    invite_id: web::Path<Uuid>,
	authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanInvite::accept_invite(&mut conn, *invite_id, authenticated)
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}


#[utoipa::path(
    post,
    summary = "[Auth]Reject clan invite",
    description = "Rejects an invite to join a clan.",
    tag = "Users - Me",
    params(
        ("invite_id" = Uuid, description = "The internal UUID of the invite")
    ),
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("/{invite_id}/reject", wrap="UserAuth::load()")]
async fn reject(
    db: web::Data<Arc<DbAppState>>,
    invite_id: web::Path<Uuid>,
	authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanInvite::reject_invite(&mut conn, *invite_id, authenticated)
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            ClanInviteResolved
        )
    ),
    paths(
        list,
		accept,
		reject
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/invites")
            .service(list)
            .service(accept)
			.service(reject)
    );
}