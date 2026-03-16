use crate::{
    app_data::db::DbAppState,
    arepl::levels::notes::{
        LevelNotePost, LevelNoteUpdate, LevelNotes, LevelNotesQueryOptions, LevelNotesResolvedPage,
        LevelNotesType,
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
    summary = "[AuthPublic]List Notes",
    description = "List all notes for a level",
    tag = "AREDL (P) - Level Notes",
    responses(
        (status = 200, body = LevelNotesResolvedPage)
    ),
    security(
        (),
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the notes list to fetch."),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page."),
        ("level_id" = Option<Uuid>, Query, description = "The internal ID of the original level to filter by."),
        ("type_filter" = Option<LevelNotesType>, Query, description = "The type of notes to filter by."),
        ("added_by" = Option<Uuid>, Query, description = "Filter by the moderator that added a note."),
    ),
)]
#[get(
    "",
    wrap = "CacheController::auth_public_with_max_age(900)",
    wrap = "UserAuth::load()"
)]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelNotesQueryOptions>,
    page_query: web::Query<PageQuery<50>>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        LevelNotes::find_all(
            &mut db.connection()?,
            query.into_inner(),
            page_query.into_inner(),
            authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add Note",
    description = "Add a note to a level",
    tag = "AREDL (P) - Level Notes",
    params(
        ("level_id" = Uuid, description = "The internal ID of the level")
    ),
    responses(
        (status = 200, body = LevelNotes)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[post("/{level_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNotePost>,
    level_id: web::Path<Uuid>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        LevelNotes::create(
            &mut db.connection()?,
            body.into_inner(),
            level_id.into_inner(),
            auth,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Note",
    description = "Update a note's info",
    tag = "AREDL (P) - Level Notes",
    params(
        ("note_id" = Uuid, description = "The internal ID of this note")
    ),
    responses(
        (status = 200, body = LevelNotes)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[patch("/{note_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNoteUpdate>,
    note_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        LevelNotes::update(
            &mut db.connection()?,
            body.into_inner(),
            note_id.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Note",
    description = "Deletes a level note",
    tag = "AREDL (P) - Level Notes",
    params(
        ("note_id" = Uuid, description = "The internal ID of this note")
    ),
    responses(
        (status = 200)
    ),
    security(("access_token" = ["LevelModify"]))
)]
#[delete("/{note_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    note_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || LevelNotes::delete(&mut db.connection()?, note_id.into_inner())).await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "AREDL (P) - Level Notes",
        description = "Endpoints for fetching and managing platformer level notes on the AREDL",
    )),
    components(schemas(
        LevelNotes,
        LevelNotePost,
        LevelNoteUpdate,

    )),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/notes")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
