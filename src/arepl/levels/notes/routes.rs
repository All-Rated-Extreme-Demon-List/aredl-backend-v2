use crate::{
    app_data::db::DbAppState,
    arepl::levels::{
        id_resolver::resolve_level_id,
        notes::{
            LevelNotePost, LevelNoteUpdate, LevelNotes, LevelNotesQueryOptions, LevelNotesResolved,
            LevelNotesType,
        },
    },
    auth::{Authenticated, Permission, UserAuth},
    error_handler::ApiError,
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
    tag = "AREDL (P) - Levels (Notes)",
    responses(
        (status = 200, body = Vec<LevelNotesResolved>)
    ),
    security(
        (),
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
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
    level_id: web::Path<String>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        LevelNotes::find_all_level(
            &mut db.connection()?,
            &query.into_inner(),
            &level_id.into_inner(),
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
    tag = "AREDL (P) - Levels (Notes)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = LevelNotes)
    ),
    security(("access_token" = ["LevelNotesModify"]))
)]
#[post("", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNotePost>,
    level_id: web::Path<String>,
    auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        LevelNotes::create(conn, body.into_inner(), level_id, &auth)
    })
    .await??;
    Ok(HttpResponse::Ok().json(notes))
}

#[derive(serde::Deserialize)]
struct NotePath {
    note_id: Uuid,
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Note",
    description = "Update a note's info",
    tag = "AREDL (P) - Levels (Notes)",
    params(
        ("note_id" = Uuid, description = "The internal ID of this note")
    ),
    responses(
        (status = 200, body = LevelNotes)
    ),
    security(("access_token" = ["LevelNotesModify"]))
)]
#[patch("/{note_id}", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelNoteUpdate>,
    path: web::Path<NotePath>,
) -> Result<HttpResponse, ApiError> {
    let notes = web::block(move || {
        LevelNotes::update(&mut db.connection()?, body.into_inner(), &path.note_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(notes))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Note",
    description = "Deletes a level note",
    tag = "AREDL (P) - Levels (Notes)",
    params(
        ("note_id" = Uuid, description = "The internal ID of this note")
    ),
    responses(
        (status = 200)
    ),
    security(("access_token" = ["LevelNotesModify"]))
)]
#[delete("/{note_id}", wrap = "UserAuth::require(Permission::LevelNotesModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<NotePath>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || LevelNotes::delete(&mut db.connection()?, &path.note_id)).await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "AREDL (P) - Levels (Notes)",
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
        web::scope("/{level_id}/notes")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
