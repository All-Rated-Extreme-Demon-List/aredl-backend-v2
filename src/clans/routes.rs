use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::clans::{members, Clan, ClanCreate, ClanListQueryOptions, ClanPage, ClanUpdate};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::clan_members;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use diesel::dsl::count_star;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Get clans",
    description = "Get paginated list of clans",
    tag = "Clans",
    params(
        ("page" = Option<i64>, Query, description = "The page of the clans list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of clans to fetch per page"),
        ("name_filter" = Option<String>, Query, description = "The search filter to apply. Uses the SQL LIKE operator syntax."),
    ),
    responses(
        (status = 200, body = Paginated<ClanPage>)
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<ClanListQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        Clan::find(
            &mut db.connection()?,
            options.into_inner(),
            page_query.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Create new clan",
    description = "Creates a new clan and sets you as owner of it. You must not be in a clan already to create one.",
    tag = "Clans",
    request_body = ClanCreate,
    responses(
        (status = 200, body = Clan)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("", wrap = "UserAuth::load()")]
async fn create_and_join(
    db: web::Data<Arc<DbAppState>>,
    clan: web::Json<ClanCreate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&clan));
    let result = web::block(move || {
        Clan::create_and_join(&mut db.connection()?, clan.into_inner(), authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create empty clan",
    description = "Creates a new empty clan. (Staff only)",
    tag = "Clans",
    request_body = ClanCreate,
    responses(
        (status = 200, body = Clan)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("/placeholder", wrap = "UserAuth::require(Permission::ClanModify)")]
async fn create_empty(
    db: web::Data<Arc<DbAppState>>,
    clan: web::Json<ClanCreate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&clan));
    let result =
        web::block(move || Clan::create_empty(&mut db.connection()?, clan.into_inner())).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Update clan",
    description = "Edit a clan's base information. You either need to be the owner of the clan or have the `ClanModify` staff permission",
    tag = "Clans",
    params(
        ("id" = Uuid, description = "The internal UUID of the clan")
    ),
    request_body = ClanUpdate,
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
#[patch("/{id}", wrap = "UserAuth::load()")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    clan: web::Json<ClanUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&clan));
    let clan_id = id.into_inner();
    let result = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_has_clan_permission(conn, clan_id, 2)?;
        Clan::update(conn, clan_id, clan.into_inner())
    })
    .await??;

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
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let clan_id = id.into_inner();
    let result = web::block(move || {
        let conn = &mut db.connection()?;

        authenticated.ensure_has_clan_permission(conn, clan_id, 2)?;
        let has_staff_permission = authenticated.has_permission(conn, Permission::ClanModify)?;

        let members_count: i64 = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .select(count_star())
            .first(conn)?;
        if members_count > 1 && !has_staff_permission {
            return Err(ApiError::new(
                403,
                "You cannot delete a clan unless you're the only member left in it.".into(),
            ));
        }

        Clan::delete(conn, clan_id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/{clan_id}/members", api = members::ApiDoc)
    ),
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
        create_and_join,
        create_empty,
        update,
		delete
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/clans")
            .configure(members::init_routes)
            .service(list)
            .service(create_and_join)
            .service(create_empty)
            .service(update)
            .service(delete),
    );
}
