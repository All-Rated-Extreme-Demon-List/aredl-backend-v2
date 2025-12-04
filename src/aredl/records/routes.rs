use crate::app_data::db::DbAppState;
use crate::aredl::records::model::{RecordInsert, RecordPatch};
use crate::aredl::records::{
    statistics, Record, RecordsQueryOptions, ResolvedRecord, ResolvedRecordPage,
};
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Staff]Get record",
    description = "Fetch details of a specific record",
    tag = "AREDL - Records",
    responses(
        (status = 200, body = ResolvedRecord),
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[get("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let record =
        web::block(move || ResolvedRecord::find(&mut db.connection()?, id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create record",
    description = "Create a new record",
    tag = "AREDL - Records",
    request_body = RecordInsert,
    responses(
        (status = 200, body = Record)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    record: web::Json<RecordInsert>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&record));
    let record = web::block(move || {
        Record::create(&mut db.connection()?, record.into_inner(), authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit record",
    description = "Edit a specific record",
    tag = "AREDL - Records",
    request_body = RecordPatch,
    params(
        ("id" = Uuid, description = "Internal record UUID")
    ),
    responses(
        (status = 200, body = Record)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    record: web::Json<RecordPatch>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&record));
    let record = web::block(move || {
        Record::update(
            &mut db.connection()?,
            id.into_inner(),
            record.into_inner(),
            authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete record",
    description = "Remove a specific record from this level",
    tag = "AREDL - Records",
    params(
        ("id" = Uuid, description = "Internal record UUID")
    ),
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let record =
        web::block(move || Record::delete(&mut db.connection()?, id.into_inner(), authenticated))
            .await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    get,
    summary = "[Staff]List records",
    description = "List a possibly filtered list of all records, with resolved levels and users data",
    tag = "AREDL - Records",
    params(
        ("page" = Option<i64>, Query, description = "The page of the list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("level_filter" = Option<Uuid>, Query, description = "The level internal UUID to filter by"),
        ("mobile_filter" = Option<bool>, Query, description = "Whether to show only/hide mobile records"),
        ("submitter_filter" = Option<String>, Query, description = "The submitter user (UUID, discord ID, or username) to filter by"),
    ),
    responses(
        (status = 200, body = Paginated<ResolvedRecordPage>)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[get("", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<RecordsQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        ResolvedRecord::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            options.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[utoipa::path(
    get,
    summary = "[Auth]List my records",
    description = "List all of the authenticated user's records",
    tag = "AREDL - Records",
    responses(
        (status = 200, body = [ResolvedRecordPage])
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
    ),
    security(
        ("access_token" = [""]),
        ("api_key" = [""]),
    )
)]
#[get("/@me", wrap = "UserAuth::load()")]
async fn find_me(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        ResolvedRecord::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            RecordsQueryOptions {
                level_filter: None,
                mobile_filter: None,
                submitter_filter: Some(authenticated.user_id.to_string()),
            },
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Records", description = "Endpoints for fetching and managing records")
    ),
    components(
        schemas(
            Record,
            RecordPatch,
            ResolvedRecord,
            ResolvedRecordPage
        )
    ),
    nest(
        (path = "/statistics", api=statistics::ApiDoc)
    ),
    paths(
        create,
        update,
        delete,
        find,
        find_all,
        find_me,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/records")
            .configure(statistics::init_routes)
            .service(create)
            .service(update)
            .service(delete)
            .service(find_all)
            .service(find_me)
            .service(find),
    );
}
