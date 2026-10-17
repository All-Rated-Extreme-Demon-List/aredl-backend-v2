use crate::list::{levels::routes::LevelPath, List};
use crate::{
    app_data::db::DbAppState,
    audit::AuditLogEntry,
    auth::{Authenticated, Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    list::levels::{
        id_resolver::resolve_level_id,
        notes::{
            LevelNotePost, LevelNoteUpdate, LevelNotes, LevelNotesQueryOptions, LevelNotesResolved,
            LevelNotesType,
        },
    },
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
    tag = "List - Levels (Notes)",
    responses(
        (status = 200, body = Vec<LevelNotesResolved>),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
    ),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("type_filter" = Option<LevelNotesType>, Query, description = "The type of notes to filter by."),
        ("added_by" = Option<Uuid>, Query, description = "Filter by the moderator that added a note."),
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "",
    wrap = "CacheController::auth_public_with_max_age(900)",
    wrap = "UserAuth::load()"
)]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelNotesQueryOptions>,
    path: web::Path<LevelPath>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let notes = db
        .run(move |conn| {
            LevelNotes::find_all_level(
                conn,
                &query.into_inner(),
                path.list,
                &path.level_id,
                authenticated,
            )
        })
        .await?;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add Note",
    description = "Add a note to a level",
    tag = "List - Levels (Notes)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 201, body = LevelNotes),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelNotesModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNotePost>,
    path: web::Path<LevelPath>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let notes = db
        .transaction(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            let notes = LevelNotes::create(conn, body.into_inner(), path.list, level_id, &auth)?;
            AuditLogEntry::log_create(conn, Some(auth.user_id), &notes)?;
            Ok::<_, ApiError>(notes)
        })
        .await?;
    Ok(HttpResponse::Created().json(notes))
}

#[derive(serde::Deserialize)]
struct NotePath {
    list: List,
    level_id: String,
    note_id: Uuid,
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Note",
    description = "Update a note's info",
    tag = "List - Levels (Notes)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("note_id" = Uuid, Path, description = "The internal ID of this note")
    ),
    responses(
        (status = 200, body = LevelNotes),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Note not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelNotesModify"])),
)]
#[patch("/{note_id}", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNoteUpdate>,
    path: web::Path<NotePath>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let notes = db
        .transaction(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            let before = LevelNotes::find_for_update(conn, path.list, level_id, path.note_id)?;
            let after =
                LevelNotes::update(conn, body.into_inner(), path.list, level_id, &path.note_id)?;
            AuditLogEntry::log_update(conn, Some(auth.user_id), &before, &after)?;
            Ok::<_, ApiError>(after)
        })
        .await?;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Note",
    description = "Deletes a level note",
    tag = "List - Levels (Notes)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("note_id" = Uuid, Path, description = "The internal ID of this note")
    ),
    responses(
        (status = 204),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
    ),
    security(("bearer_token" = ["LevelNotesModify"])),
)]
#[delete("/{note_id}", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<NotePath>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    db.transaction(move |conn| {
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        if let Some(deleted) = LevelNotes::delete(conn, path.list, level_id, &path.note_id)? {
            AuditLogEntry::log_delete(conn, Some(auth.user_id), &deleted)?;
        }
        Ok::<_, ApiError>(())
    })
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "List - Levels (Notes)",
        description = "Endpoints for fetching and managing level notes on the AREDL",
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
        web::scope("/{level_id}/notes")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
