use std::sync::Arc;
use uuid::Uuid;
use actix_web::{get, patch, post, delete, web, HttpResponse};
use utoipa::OpenApi;
use diesel::{QueryDsl, RunQueryDsl, ExpressionMethods, OptionalExtension, SelectableHelper};
use diesel::dsl::count_star;
use crate::auth::{UserAuth, Permission, Authenticated};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::page_helper::{PageQuery, Paginated};
use crate::clans::{Clan, ClanCreate, ClanListQueryOptions, ClanMember, ClanPage, ClanUpdate};
use crate::schema::clan_members;

#[utoipa::path(
    get,
    summary = "Get clan members",
    description = "Get the list of members of a certain clan",
    tag = "Clans - Users",
	params(
		("id" = Uuid, Path, description = "The internal UUID of the clan")
	),
    responses(
        (status = 200, body = [BaseUser])
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
	id: web::Path<Uuid>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        BaseUser::find_all_clan_members(&mut conn, id.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add clan member",
    description = "Adds a member to a clan",
    tag = "Clans - Users",
    request_body = ClanCreate,
    responses(
        (status = 200, body = Clan)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("", wrap="UserAuth::load()")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    clan: web::Json<ClanCreate>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        Clan::create(&mut conn, clan.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete clan",
    description = "Remove a clan. You either need to be the owner of the clan or have the `ClanModify` staff permission, and the clan needs to be empty (except for the owner)",
    tag = "Clans",
    params(
        ("id" = Uuid, description = "The internal UUID of the clan")
    ),
    responses(
        (status = 200, body = Clan)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
		("clan_id" = ["ClanModify"]),
		("api_key" = ["ClanModify"]),
    )
)]
#[delete("/{id}", wrap = "UserAuth::load()")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated, ) -> Result<HttpResponse, ApiError> {
	let clan_id = id.into_inner();
	let result = web::block(move || {
		let mut conn = db.connection()?;

		let member = clan_members::table
			.filter(clan_members::clan_id.eq(clan_id))
			.filter(clan_members::user_id.eq(authenticated.user_id))
			.select(ClanMember::as_select())
			.first::<ClanMember>(&mut conn)
			.optional()?;

		let has_permission = authenticated.has_permission(db, Permission::ClanModify)?;
		if (member.is_none() || member.unwrap().role < 2 ) && !has_permission {
			return Err(ApiError::new(403, "You need to own this clan to be able to delete it.".into()));
		}

		let members_count: i64 = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .select(count_star())
            .first(&mut conn)?;
        if members_count > 1 && !has_permission {
            return Err(ApiError::new(403, "You cannot delete a clan unless you're the only member left in it.".into()));
        }

		Clan::delete(&mut conn, clan_id)
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
			ClanPage
        )
    ),
    paths(
        list,
        create,
        update,
		delete
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/clans")
            .service(list)
            .service(create)
            .service(update)
			.service(delete)
    );
}