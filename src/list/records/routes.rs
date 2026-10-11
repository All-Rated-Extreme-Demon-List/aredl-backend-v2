use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::levels::id_resolver::resolve_level_id;
use crate::list::records::model::{RecordInsert, RecordSortField, ResolvedRecordPage};
use crate::list::records::{
    MutualVictors, MutualVictorsQuery, Record, RecordPatch, RecordsQueryOptions, ResolvedRecord,
};
use crate::list::List;
use crate::page_helper::{PageQuery, Paginated};
use crate::providers::ProvidersAppState;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
struct RecordPath {
    pub list: List,
    pub id: Uuid,
}

#[utoipa::path(
    get,
    summary = "[Staff]Get record",
    description = "Fetch details of a specific record",
    tag = "List - Records",
    responses(
        (status = 200, body = ResolvedRecord),
        (status = 404, description = "Record not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["RecordModify"])),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal record UUID"),
    ),
)]
#[get("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<RecordPath>,
) -> Result<HttpResponse, ApiError> {
    let record =
        web::block(move || ResolvedRecord::find(&mut db.connection()?, path.list, path.id))
            .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create record",
    description = "Create a new record",
    tag = "List - Records",
    request_body = RecordInsert,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 200, body = Record),
        (status = 403, description = "You cannot create records for yourself", body = ErrorResponse, examples(
            ("own_record" = (value = json!({"message": "You cannot create records for yourself"})))
        ))
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    record: web::Json<RecordInsert>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&record));
    let record = web::block(move || {
        Record::create(
            &mut db.connection()?,
            *list,
            &record.into_inner(),
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit record",
    description = "Edit a specific record",
    tag = "List - Records",
    request_body = RecordPatch,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal record UUID")
    ),
    responses(
        (status = 200, body = Record),
        (status = 403, description = "You cannot update records for yourself", body = ErrorResponse, examples(
            ("own_record" = (value = json!({"message": "You cannot update records for yourself"})))
        )),
        (status = 404, description = "Record not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<RecordPath>,
    record: web::Json<RecordPatch>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&record));
    let record = web::block(move || {
        Record::update(
            &mut db.connection()?,
            path.list,
            path.id,
            &record.into_inner(),
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update record completion timestamp",
    description = "Tries to fetch and update the achieved_at timestamp for this record, by looking up the completion video's link",
    tag = "List - Records",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal record UUID")
    ),
    responses(
        (status = 200, body = Record),
        (status = 404, description = "Record not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[patch(
    "/{id}/update-timestamp",
    wrap = "UserAuth::require(Permission::RecordModify)"
)]
async fn update_timestamp(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<RecordPath>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let record = Record::update_timestamp(db, path.list, path.id, providers.get_ref()).await?;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete record",
    description = "Remove a specific record from this level",
    tag = "List - Records",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal record UUID")
    ),
    responses(
        (status = 204),
        (status = 404, description = "Record not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<RecordPath>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || Record::delete(&mut db.connection()?, path.list, path.id, &authenticated))
        .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    summary = "[Staff]List mutual victors",
    description = "List users who have beaten both levels",
    tag = "List - Records",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Query, description = "First level ID (internal UUID, list position, or GD ID; add _2p for the two-player GD ID variant)"),
        ("other_level_id" = String, Query, description = "Second level ID (internal UUID, list position, or GD ID; add _2p for the two-player GD ID variant)"),
        ("high_extremes" = Option<bool>, Query, description = "Whether to show only users with more than 50 records on the selected list"),
    ),
    responses(
        (status = 200, body = MutualVictors),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "One of the levels was not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[get(
    "/mutual-victors",
    wrap = "UserAuth::require(Permission::RecordModify)",
    wrap = "CacheController::public_with_max_age(900)"
)]
async fn mutual_victors(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    query: web::Query<MutualVictorsQuery>,
) -> Result<HttpResponse, ApiError> {
    let query = query.into_inner();
    let victors = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, *list, &query.level_id)?;
        let other_level_id = resolve_level_id(conn, *list, &query.other_level_id)?;
        MutualVictors::find(conn, *list, level_id, other_level_id, query.high_extremes)
    })
    .await??;
    Ok(HttpResponse::Ok().json(victors))
}

#[utoipa::path(
    get,
    summary = "[Staff]List records",
    description = "List a possibly filtered list of all records, with resolved levels and users data",
    tag = "List - Records",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("verification_filter" = Option<bool>, Query, description = "Whether to show only/hide verification records"),
        ("sort" = Option<RecordSortField>, Query, description = "The sorting type to use. Completion-time sorting is only available for platformer."),
        PageQuery<100>,
        ("level_filter" = Option<Uuid>, Query, description = "The level internal UUID to filter by"),
        ("mobile_filter" = Option<bool>, Query, description = "Whether to show only/hide mobile records"),
        ("submitter_filter" = Option<String>, Query, description = "The submitter user (UUID, discord ID, or username) to filter by"),
    ),
    responses(
        (status = 200, body = Paginated<ResolvedRecordPage>),
    ),
    security(("bearer_token" = ["RecordModify"])),
)]
#[get("", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<RecordsQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        ResolvedRecord::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            *list,
            &options.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[utoipa::path(
    get,
    summary = "[Auth]List my records",
    description = "List all of the authenticated user's records",
    tag = "List - Records",
    responses(
        (status = 200, body = Paginated<ResolvedRecordPage>),
    ),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        PageQuery<100>,
    ),
    security(("bearer_token" = [])),
)]
#[get("/@me", wrap = "UserAuth::load()")]
async fn find_me(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    page_query: web::Query<PageQuery<100>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        ResolvedRecord::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            *list,
            &RecordsQueryOptions {
                level_filter: None,
                mobile_filter: None,
                verification_filter: None,
                submitter_filter: Some(authenticated.user_id.to_string()),
                sort: None,
            },
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Records", description = "Endpoints for fetching and managing records")
    ),
    components(
        schemas(
            Record,
            RecordSortField,
            MutualVictors,
            RecordPatch,
            ResolvedRecord,
        )
    ),
    paths(
        create,
        update,
        update_timestamp,
        delete,
        find,
        find_all,
        find_me,
        mutual_victors,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/records")
            .service(create)
            .service(update)
            .service(update_timestamp)
            .service(delete)
            .service(find_all)
            .service(find_me)
            .service(mutual_victors)
            .service(find),
    );
}
