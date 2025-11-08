use crate::{
    aredl::levels::ldms::{
        LevelLDM, LevelLDMBody, LevelLDMQueryOptions, LevelLDMStatus, LevelLDMType, LevelLDMUpdate,
    },
    auth::{Authenticated, Permission, UserAuth},
    db::DbAppState,
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
    summary = "List LDMs",
    description = "List all LDMs for a level",
    tag = "AREDL - Level LDMs",
    responses(
        (status = 200, body = [LevelLDM])
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the LDM list to fetch."),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page."),
        ("level_id" = Option<Uuid>, Query, description = "The internal ID of the original level to filter by."),
        ("type_filter" = Option<LevelLDMType>, Query, description = "The type of LDM to filter by."),
        ("status_filter" = Option<LevelLDMStatus>, Query, description = "The status of an LDM to filter by."),
        ("description" = Option<String>, Query, description = "Filter for the description of this LDM. Use SQL LIKE syntax."),
        ("added_by" = Option<Uuid>, Query, description = "Filter by the moderator that added an LDM."),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelLDMQueryOptions>,
    page_query: web::Query<PageQuery<50>>,
) -> Result<HttpResponse, ApiError> {
    let ldms = web::block(move || {
        LevelLDM::find_all(
            &mut db.connection()?,
            query.into_inner(),
            page_query.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(ldms))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add LDM",
    description = "Add an LDM to a level",
    tag = "AREDL - Level LDMs",
    params(
        ("level_id" = Uuid, description = "The internal ID of the level")
    ),
    responses(
        (status = 200, body = LevelLDM)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[post("/{level_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelLDMBody>,
    level_id: web::Path<Uuid>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let ldms = web::block(move || {
        LevelLDM::create(
            &mut db.connection()?,
            body.into_inner(),
            level_id.into_inner(),
            auth,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(ldms))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update LDM",
    description = "Update an LDM's info",
    tag = "AREDL - Level LDMs",
    params(
        ("ldm_id" = Uuid, description = "The internal ID of this LDM")
    ),
    responses(
        (status = 200, body = LevelLDM)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[patch("/{ldm_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelLDMUpdate>,
    ldm_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let ldms = web::block(move || {
        LevelLDM::update(
            &mut db.connection()?,
            body.into_inner(),
            ldm_id.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(ldms))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete LDM",
    description = "Deletes an LDM",
    tag = "AREDL - Level LDMs",
    params(
        ("ldm_id" = Uuid, description = "The internal ID of this LDM")
    ),
    responses(
        (status = 200)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[delete("/{ldm_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    ldm_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || LevelLDM::delete(&mut db.connection()?, ldm_id.into_inner())).await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "AREDL - Level LDMs",
        description = "Endpoints for fetching and managing level LDMs on the AREDL",
    )),
    components(schemas(
        LevelLDM,
        LevelLDMBody,
        LevelLDMUpdate,

    )),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/ldms")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
