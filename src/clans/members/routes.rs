use crate::auth::{Authenticated, Permission, UserAuth};
use crate::clans::members::ClanMemberResolved;
use crate::clans::members::{ClanInviteCreate, ClanMemberInvite, ClanMemberUpdate};
use crate::clans::{
    Clan, ClanCreate, ClanInvite, ClanListQueryOptions, ClanMember, ClanPage, ClanUpdate,
};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Get clan members",
    description = "Get the list of members of a certain clan",
    tag = "Clans - Members",
	params(
		("clan_id" = Uuid, Path, description = "The internal UUID of the clan")
	),
    responses(
        (status = 200, body = [ClanMemberResolved])
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::find_all_clan_members(&mut conn, clan_id.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add clan members",
    description = "Add member(s) to a clan",
    tag = "Clans - Members",
    request_body = [Uuid],
    params(
        ("clan_id" = Uuid, description = "The internal UUID of the clan")
    ),
    responses(
        (status = 200, body = [Uuid])
    ),
    security(
        ("access_token" = ["ClanModify"]),
        ("api_key" = ["ClanModify"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::ClanModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    members: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&members));
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::add_all(&mut conn, *clan_id, members.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Set clan members",
    description = "Sets the members of a clan. This will remove all existing members and add the new ones.",
    tag = "Clans - Members",
    request_body = [Uuid],
    params(
        ("clan_id" = Uuid, description = "The internal UUID of the clan")
    ),
    responses(
        (status = 200, body = [Uuid])
    ),
    security(
        ("access_token" = ["ClanModify"]),
        ("api_key" = ["ClanModify"]),
    )
)]
#[patch("", wrap = "UserAuth::require(Permission::ClanModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    members: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&members));
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::set_all(&mut conn, *clan_id, members.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Remove clan members",
    description = "Remove members from a clan. You either need to be the owner/vice owner of the clan or have the `ClanModify` staff permission.",
    tag = "Clans - Members",
    params(
        ("clan_id" = Uuid, description = "The internal UUID of the clan")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = Clan)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
		("access_token" = ["ClanModify"]),
		("api_key" = ["ClanModify"]),
    )
)]
#[delete("", wrap = "UserAuth::load()")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    members: web::Json<Vec<Uuid>>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&members));
    let clan_id = clan_id.into_inner();
    let result = web::block(move || {
        let mut conn = db.connection()?;
        authenticated.has_clan_permission(db.clone(), clan_id, 1)?;

        for member_id in members.iter() {
            authenticated.has_clan_higher_permission(db.clone(), clan_id, *member_id)?;
        }

        ClanMember::remove_all(&mut conn, clan_id, members.into_inner())
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Invite member",
    description = "Invite a user to join a clan",
    tag = "Clans - Members",
    request_body = ClanMemberInvite,
    params(
        ("clan_id" = Uuid, description = "The internal UUID of the clan")
    ),
    responses(
        (status = 200, body = ClanInvite)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
		("access_token" = ["ClanModify"]),
		("api_key" = ["ClanModify"]),
    )
)]
#[post("/invite", wrap = "UserAuth::load()")]
async fn invite(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    user: web::Json<ClanMemberInvite>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&user));
    let result = web::block(move || {
        let mut conn = db.connection()?;

        authenticated.has_clan_permission(db.clone(), *clan_id, 1)?;

        let invite = ClanInvite::create(
            &mut conn,
            ClanInviteCreate {
                clan_id: *clan_id,
                user_id: user.user_id,
                invited_by: authenticated.user_id,
            },
        )?;

        Ok::<ClanInvite, ApiError>(invite)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit member",
    description = "Changes a member's role in a clan",
    tag = "Clans - Members",
    request_body = ClanMemberUpdate,
    params(
        ("clan_id" = Uuid, description = "The internal UUID of the clan"),
        ("user_id" = Uuid, description = "The internal UUID of the user")
    ),
    request_body = ClanMemberUpdate,
    responses(
        (status = 200, body = ClanMember)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
		("access_token" = ["ClanModify"]),
		("api_key" = ["ClanModify"]),
    )
)]
#[patch("/{user_id}", wrap = "UserAuth::load()")]
async fn edit(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<(Uuid, Uuid)>,
    member: web::Json<ClanMemberUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&member));
    let (clan_id, user_id) = path.into_inner();
    let result = web::block(move || {
        let mut conn = db.connection()?;

        authenticated.has_clan_permission(db.clone(), clan_id, 2)?;

        let member =
            ClanMember::edit_member_role(&mut conn, clan_id, user_id, member.into_inner())?;

        Ok::<ClanMember, ApiError>(member)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        Clan,
        ClanCreate,
        ClanUpdate,
        ClanListQueryOptions,
        ClanPage,
        ClanInvite,
        ClanMemberInvite,
        ClanMemberResolved
    )),
    paths(list, add, set, delete, invite, edit)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{clan_id}/members")
            .service(list)
            .service(add)
            .service(set)
            .service(delete)
            .service(invite)
            .service(edit),
    );
}
