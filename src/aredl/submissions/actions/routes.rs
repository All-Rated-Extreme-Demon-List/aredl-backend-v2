use crate::{
    aredl::{
        records::Record,
        submissions::{
            actions::{AcceptParams, ReviewerNotes},
            Submission, SubmissionResolved,
        },
    },
    auth::{Authenticated, Permission, UserAuth},
    db::DbAppState,
    error_handler::ApiError,
    notifications::WebsocketNotification,
};
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Staff]Claim a submission",
    description = "Claim the submission with the highest priority to be checked.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("claim", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn claim(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let patched = web::block(move || {
        SubmissionResolved::claim_highest_priority(&mut db.connection()?, authenticated)
    })
    .await??;

    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    post,
    summary = "[Staff]Unclaim a submission",
    description = "Unclaim a submission you have previously claimed.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post(
    "{id}/unclaim",
    wrap = "UserAuth::require(Permission::SubmissionReview)"
)]
async fn unclaim(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let patched = web::block(move || {
        Submission::unclaim(&mut db.connection()?, id.into_inner(), authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    post,
    summary = "[Staff]Accept a submission",
    description = "Accept a submission you have previously claimed, adding it as a record to the site.",
    tag = "AREDL - Submissions",
    responses(
        (status = 202, body = Record)
    ),
    request_body = AcceptParams,
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post(
    "{id}/accept",
    wrap = "UserAuth::require(Permission::SubmissionReview)"
)]
async fn accept(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
    opts: web::Json<AcceptParams>,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&opts));
    let new_record = web::block(move || {
        Submission::accept(
            &mut db.connection()?,
            notify_tx.get_ref().clone(),
            id.into_inner(),
            authenticated.user_id,
            opts.into_inner(),
        )
    })
    .await??;

    Ok(HttpResponse::Accepted().json(new_record))
}

#[utoipa::path(
    post,
    summary = "[Staff]Deny a submission",
    description = "Deny a submission you have previously claimed, adding it as a record to the site.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    request_body = ReviewerNotes,
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post("{id}/deny", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn deny(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
    notes: web::Json<ReviewerNotes>,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&notes));

    let new_record = web::block(move || {
        Submission::reject(
            &mut db.connection()?,
            notify_tx.get_ref().clone(),
            id.into_inner(),
            authenticated,
            notes.into_inner().notes,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(new_record))
}

#[utoipa::path(
    post,
    summary = "[Staff]Place a submission under consideration",
    description = "Set a submission's status to under consideration.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post(
    "{id}/underconsideration",
    wrap = "UserAuth::require(Permission::SubmissionReview)"
)]
async fn under_consideration(
    db: web::Data<Arc<DbAppState>>,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
    notes: web::Json<ReviewerNotes>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&notes));

    let new_record = web::block(move || {
        Submission::under_consideration(
            &mut db.connection()?,
            notify_tx.get_ref().clone(),
            id.into_inner(),
            authenticated,
            notes.into_inner().notes,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(new_record))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(ReviewerNotes, Record)),
    paths(claim, unclaim, accept, deny, under_consideration,)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config
        .service(claim)
        .service(unclaim)
        .service(accept)
        .service(deny)
        .service(under_consideration);
}
