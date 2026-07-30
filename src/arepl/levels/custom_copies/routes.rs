use crate::{
    app_data::db::DbAppState,
    arepl::levels::{
        custom_copies::{
            LevelCustomCopy, LevelCustomCopyBody, LevelCustomCopyQueryOptions,
            LevelCustomCopyStatus, LevelCustomCopyType, LevelCustomCopyUpdate,
        },
        id_resolver::resolve_level_id,
    },
    auth::{Authenticated, Permission, UserAuth},
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
    summary = "List Custom Copies",
    description = "List all custom copies for a level",
    tag = "AREDL (P) - Levels (Custom Copies)",
    responses(
        (status = 200, body = [LevelCustomCopy])
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the custom copy list to fetch."),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page."),
        ("level_id" = Option<String>, Query, description = "The ID of the original level to filter by (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("type_filter" = Option<LevelCustomCopyType>, Query, description = "The type of custom copy to filter by."),
        ("status_filter" = Option<LevelCustomCopyStatus>, Query, description = "The status of a custom copy to filter by."),
        ("description" = Option<String>, Query, description = "Filter for the description of this custom copy. Use SQL LIKE syntax."),
        ("added_by" = Option<Uuid>, Query, description = "Filter by the moderator that added a custom copy."),
    )
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelCustomCopyQueryOptions>,
    page_query: web::Query<PageQuery<50>>,
) -> Result<HttpResponse, ApiError> {
    let custom_copies = web::block(move || {
        LevelCustomCopy::find_all(
            &mut db.connection()?,
            &query.into_inner(),
            page_query.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(custom_copies))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add Custom Copy",
    description = "Add a custom copy to a level",
    tag = "AREDL (P) - Levels (Custom Copies)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = LevelCustomCopy)
    ),
    security(("access_token" = ["LevelCustomCopiesModify"]))
)]
#[post(
    "/{level_id}",
    wrap = "UserAuth::require(Permission::LevelCustomCopiesModify)"
)]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelCustomCopyBody>,
    level_id: web::Path<String>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let custom_copies = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        LevelCustomCopy::create(conn, body.into_inner(), level_id, &auth)
    })
    .await??;
    Ok(HttpResponse::Ok().json(custom_copies))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Custom Copy",
    description = "Update a custom copy's info",
    tag = "AREDL (P) - Levels (Custom Copies)",
    params(
        ("copy_id" = Uuid, description = "The internal ID of this custom copy")
    ),
    responses(
        (status = 200, body = LevelCustomCopy)
    ),
    security(("access_token" = ["LevelCustomCopiesModify"]))
)]
#[patch(
    "/{copy_id}",
    wrap = "UserAuth::require(Permission::LevelCustomCopiesModify)"
)]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelCustomCopyUpdate>,
    copy_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let custom_copies = web::block(move || {
        LevelCustomCopy::update(
            &mut db.connection()?,
            body.into_inner(),
            copy_id.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(custom_copies))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Custom Copy",
    description = "Deletes a custom copy",
    tag = "AREDL (P) - Levels (Custom Copies)",
    params(
        ("copy_id" = Uuid, description = "The internal ID of this custom copy")
    ),
    responses(
        (status = 200)
    ),
    security(("access_token" = ["LevelCustomCopiesModify"]))
)]
#[delete(
    "/{copy_id}",
    wrap = "UserAuth::require(Permission::LevelCustomCopiesModify)"
)]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    copy_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || LevelCustomCopy::delete(&mut db.connection()?, copy_id.into_inner()))
        .await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "AREDL (P) - Levels (Custom Copies)",
        description = "Endpoints for fetching and managing level custom copies on the AREDL",
    )),
    components(schemas(
        LevelCustomCopy,
        LevelCustomCopyBody,
        LevelCustomCopyUpdate,

    )),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/custom-copies")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
