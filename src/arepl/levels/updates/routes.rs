use crate::{
    app_data::db::DbAppState,
    arepl::levels::{
        id_resolver::resolve_level_id,
        updates::{
            LevelUpdateEntry, LevelUpdateEntryPage, LevelUpdateEntryPost,
            LevelUpdateEntryQueryOptions, LevelUpdateEntryUpdate, LevelUpdateType,
        },
    },
    auth::{Permission, UserAuth},
    error_handler::ApiError,
    page_helper::PageQuery,
    CacheController,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "List Updates",
    description = "List all updates for a level",
    tag = "AREDL (P) - Levels (Updates)",
    responses(
        (status = 200, body = LevelUpdateEntryPage)
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the updates list to fetch."),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page."),
        ("level_id" = Option<String>, Query, description = "The ID of the original level to filter by (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("type_filter" = Option<LevelUpdateType>, Query, description = "The type of update to filter by."),
    )
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelUpdateEntryQueryOptions>,
    page_query: web::Query<PageQuery<50>>,
) -> Result<HttpResponse, ApiError> {
    let updates = web::block(move || {
        LevelUpdateEntry::find_all(
            &mut db.connection()?,
            &query.into_inner(),
            page_query.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(updates))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add Update",
    description = "Add an update to a level",
    tag = "AREDL (P) - Levels (Updates)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = LevelUpdateEntry)
    ),
    security(("access_token" = ["LevelUpdatesModify"]))
)]
#[post(
    "/{level_id}",
    wrap = "UserAuth::require(Permission::LevelUpdatesModify)"
)]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelUpdateEntryPost>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let created = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        LevelUpdateEntry::create(conn, body.into_inner(), level_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Update",
    description = "Update a level update's info",
    tag = "AREDL (P) - Levels (Updates)",
    params(
        ("update_id" = Uuid, description = "The internal ID of this update")
    ),
    responses(
        (status = 200, body = LevelUpdateEntry)
    ),
    security(("access_token" = ["LevelUpdatesModify"]))
)]
#[patch(
    "/{update_id}",
    wrap = "UserAuth::require(Permission::LevelUpdatesModify)"
)]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelUpdateEntryUpdate>,
    update_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let updated = web::block(move || {
        LevelUpdateEntry::update(
            &mut db.connection()?,
            body.into_inner(),
            update_id.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(updated))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Update",
    description = "Deletes a level update",
    tag = "AREDL (P) - Levels (Updates)",
    params(
        ("update_id" = Uuid, description = "The internal ID of this update")
    ),
    responses(
        (status = 200)
    ),
    security(("access_token" = ["LevelUpdatesModify"]))
)]
#[delete(
    "/{update_id}",
    wrap = "UserAuth::require(Permission::LevelUpdatesModify)"
)]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    update_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || LevelUpdateEntry::delete(&mut db.connection()?, update_id.into_inner()))
        .await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "AREDL (P) - Levels (Updates)",
        description = "Endpoints for fetching and managing level updates on the AREDL",
    )),
    components(schemas(
        LevelUpdateEntry,
        LevelUpdateEntryPost,
        LevelUpdateEntryUpdate,
    )),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/updates")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
