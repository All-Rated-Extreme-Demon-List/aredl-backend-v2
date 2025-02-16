use std::sync::Arc;
use uuid::Uuid;
use actix_web::{get, patch, post, delete, web, HttpResponse};
use utoipa::OpenApi;
use diesel::Connection;
use crate::auth::{UserAuth, Permission, Authenticated};
use crate::clans::members::{ClanInviteCreate, ClanMemberInvite, ClanMemberUpdate};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::clans::{Clan, ClanCreate, ClanInvite, ClanListQueryOptions, ClanMember, ClanPage, ClanUpdate};
use crate::clans::members::ClanMemberResolved;

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
	clan_id: web::Path<Uuid>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::find_all_clan_members(&mut conn, clan_id.into_inner())
    }).await??;
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
#[post("", wrap="UserAuth::require(Permission::ClanModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    members: web::Json<Vec<Uuid>>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::add_all(&mut conn, *clan_id, members.into_inner())
    }).await??;
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
#[patch("", wrap="UserAuth::require(Permission::ClanModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    members: web::Json<Vec<Uuid>>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ClanMember::set_all(&mut conn, *clan_id, members.into_inner())
    }).await??;
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
async fn delete(db: web::Data<Arc<DbAppState>>, clan_id: web::Path<Uuid>, members: web::Json<Vec<Uuid>>, authenticated: Authenticated, ) -> Result<HttpResponse, ApiError> {
	let clan_id = clan_id.into_inner();
	let result = web::block(move || {
		let mut conn = db.connection()?;
        authenticated.has_clan_permission(db.clone(), clan_id, 1)?;

        for member_id in members.iter() {
            authenticated.has_clan_higher_permission(db.clone(), clan_id, *member_id)?;
        }

        ClanMember::remove_all(&mut conn, clan_id, members.into_inner())

		}).await??;

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
#[post("/invite", wrap="UserAuth::load()")]
async fn invite(
    db: web::Data<Arc<DbAppState>>,
    clan_id: web::Path<Uuid>,
    user: web::Json<ClanMemberInvite>,
    authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;

        authenticated.has_clan_permission(db.clone(), *clan_id, 1)?;
				
		let invite = ClanInvite::create(&mut conn, ClanInviteCreate {
			clan_id: *clan_id,
			user_id: user.user_id,
			invited_by: authenticated.user_id,
		})?;

		Ok::<ClanInvite, ApiError>(invite)
		
    }).await??;
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
#[patch("/{user_id}", wrap="UserAuth::load()")]
async fn edit(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<(Uuid, Uuid)>,
    member: web::Json<ClanMemberUpdate>,
    authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let (clan_id, user_id) = path.into_inner();
    let result = web::block(move || {
        let mut conn = db.connection()?;

        authenticated.has_clan_permission(db.clone(), clan_id, 2)?;

        let result = conn.transaction(|connection| -> Result<ClanMember, ApiError> {
            let member = ClanMember::edit_member_role(connection, clan_id, user_id, member.into_inner())?;
            if member.role == 2 {
                ClanMember::edit_member_role(connection, clan_id, authenticated.user_id, ClanMemberUpdate { role: 1 })?;
            }
            Ok(member)
        })?;

        Ok::<ClanMember, ApiError>(result)
        
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            Clan,
            ClanCreate,
            ClanUpdate,
            ClanListQueryOptions,
			ClanPage,
            ClanInvite,
            ClanMemberInvite,
            ClanMemberResolved
        )
    ),
    paths(
        list,
        add,
		delete,
        invite,
        edit
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{clan_id}/members")
            .service(list)
            .service(add)
			.service(delete)
            .service(invite)
            .service(edit)
    );
}