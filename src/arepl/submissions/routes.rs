use crate::{
    app_data::db::DbAppState,
    arepl::{
        records::Record,
        submissions::{
            patch::{SubmissionPatchMod, SubmissionPatchUser},
            pemonlist,
            post::{SubmissionInsert, SubmissionPostMod},
            resolved::{ResolvedSubmissionPage, SubmissionQueryOptions, SubmissionsSortField},
            status, Submission, SubmissionPage, SubmissionResolved, SubmissionStatus,
        },
    },
    auth::{Authenticated, Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    notifications::WebsocketNotification,
    page_helper::{PageQuery, Paginated},
    providers::ProvidersAppState,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tokio::sync::broadcast;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

use super::{history, queue};

#[utoipa::path(
    get,
    summary = "[Staff]List submissions",
    description = "Get a possibly filtered list of resolved submissions.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>),
    ),
    params(
        ("sort" = Option<SubmissionsSortField>, Query, description = "The sorting type to use"),
        PageQuery<50>,
        ("level_filter" = Option<Uuid>, Query, description = "Filter submissions to a specific level UUID"),
        ("status_filter" = Option<SubmissionStatus>, Query, description = "Filter submissions to specific statuses"),
        ("mobile_filter" = Option<bool>, Query, description = "Filter submissions to mobile/desktop submissions only"),
        ("submitter_filter" = Option<String>, Query, description = "Filter submissions to a specific submitter (UUID, discord ID, or username)"),
        ("priority_filter" = Option<bool>, Query, description = "Filter submissions to priority/non-priority submissions"),
        ("reviewer_filter" = Option<String>, Query, description = "Filter submissions to a specific reviewer (UUID, discord ID, or username)"),
        ("note_filter" = Option<String>, Query, description = "Filter submissions that contain a specific note substring"),
),
    security(("bearer_token" = ["SubmissionReview"])),
)]
#[get("", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            options.into_inner(),
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get a resolved submission",
    description = "Get a specific submission by its ID. If you aren't staff, the submission must be yours.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = SubmissionResolved),
        (status = 404, description = "Submission not found or not visible to this user", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
    security(("bearer_token" = [])),
)]
#[get("/{id}", wrap = "UserAuth::load()")]
async fn find_one(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submission = web::block(move || {
        SubmissionResolved::find_one(&mut db.connection()?, id.into_inner(), &authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get own submissions",
    description = "List all submissions submitted by the logged in user.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>),
    ),
    params(
        ("sort" = Option<SubmissionsSortField>, Query, description = "The sorting type to use"),
        ("priority_filter" = Option<bool>, Query, description = "Filter submissions to priority/non-priority submissions"),
        ("note_filter" = Option<String>, Query, description = "Filter submissions that contain a specific note substring"),
        PageQuery<50>,
        ("level_filter" = Option<Uuid>, Query, description = "Filter submissions to a specific level UUID"),
        ("status_filter" = Option<SubmissionStatus>, Query, description = "Filter submissions to specific statuses"),
        ("mobile_filter" = Option<bool>, Query, description = "Filter submissions to mobile/desktop submissions only"),
),
    security(("bearer_token" = [])),
)]
#[get("/@me", wrap = "UserAuth::load()")]
async fn find_me(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_own(
            &mut db.connection()?,
            page_query.into_inner(),
            options.into_inner(),
            &authenticated,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    post,
    summary = "[Auth]Create a submission",
    description = "Create a submission to be checked by a reviewer.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 201, body = Submission),
        (status = 403, description = "You have been banned from the list, or submissions are currently disabled", body = ErrorResponse, examples(
            ("banned" = (value = json!({"message": "You have been banned from the list."}))),
            ("disabled" = (value = json!({"message": "Submissions are currently disabled"})))
        )),
        (status = 404, description = "User or level not found", body = ErrorResponse),
        (status = 409, description = "You already have a submission for this level", body = ErrorResponse),
        (status = 422, description = "This level is on the legacy list, or required raw footage is missing", body = ErrorResponse, examples(
            ("legacy_level" = (value = json!({"message": "This level is on the legacy list and is not accepting records."}))),
            ("raw_footage_required" = (value = json!({"message": "Platformer submissions require raw footage"})))
        ))
    ),
    request_body = SubmissionPostMod,
    security(("bearer_token" = [])),
)]
#[post("", wrap = "UserAuth::load()")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<SubmissionPostMod>,
    authenticated: Authenticated,
    root_span: RootSpan,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let created = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_not_banned(conn)?;
        Submission::create(conn, body.into_inner(), &authenticated, providers.as_ref())
    })
    .await??;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit a submission",
    description = "Edit a submission. If you aren't staff, the submission must be yours and not being actively reviewed.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Submission),
        (status = 403, description = "User is banned, submission is locked or belongs to another user, or reviewer permissions are insufficient", body = ErrorResponse, examples(
            ("banned" = (value = json!({"message": "You have been banned from submitting records."}))),
            ("not_submitter" = (value = json!({"message": "You can only edit your own submissions."}))),
            ("locked" = (value = json!({"message": "This submission has been locked and cannot be edited"}))),
            ("reviewer_permissions" = (value = json!({"message": "You do not have permission to edit this submission."})))
        )),
        (status = 404, description = "User, submission or level not found", body = ErrorResponse),
        (status = 409, description = "This submission is currently being reviewed and cannot be edited", body = ErrorResponse),
        (status = 422, description = "This level is on the legacy list and is not accepting records", body = ErrorResponse)
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
    request_body = SubmissionPatchMod,
    security(("bearer_token" = [])),
)]
#[patch("/{id}", wrap = "UserAuth::load()")]
async fn patch(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    body: web::Json<SubmissionPatchMod>,
    authenticated: Authenticated,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let db_clone = db.clone();
    let providers_clone = providers.clone();
    let patched = web::block(move || {
        let conn = &mut db.connection()?;
        if authenticated.has_permission(conn, Permission::SubmissionReview)? {
            SubmissionPatchMod::patch(
                body.into_inner(),
                id.into_inner(),
                conn,
                &authenticated,
                &notify_tx,
                &providers,
            )
        } else {
            let user_patch = SubmissionPatchMod::downgrade(body.into_inner());
            SubmissionPatchUser::patch(
                user_patch,
                id.into_inner(),
                conn,
                &authenticated,
                &providers,
            )
        }
    })
    .await??;

    // if the status submission is changed to accepted, trigger other actions (timestamp update, badges, bounties, etc)
    if patched.status == SubmissionStatus::Accepted {
        Record::post_accept_actions(db_clone, &patched, providers_clone);
    }
    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    get,
    summary = "[Staff]Claim a submission",
    description = "Claim the next submission to be checked. Alternates between priority and non-priority submissions when possible.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = SubmissionResolved),
        (status = 404, description = "There are no submissions available to claim", body = ErrorResponse)
    ),
    security(("bearer_token" = ["SubmissionReview"])),
)]
#[get("/claim", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn claim(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let patched = web::block(move || {
        Submission::claim_highest_priority(&mut db.connection()?, &authenticated)
    })
    .await??;

    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete a submission",
    description = "Delete a submission by its ID. If you aren't staff, the submission must be yours and in the pending state.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 204),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
    security(("bearer_token" = [])),
)]
#[delete("/{id}", wrap = "UserAuth::load()")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || Submission::delete(&mut db.connection()?, id.into_inner(), &authenticated))
        .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL (P) - Submissions", description = "Endpoints for fetching and managing submissions")
    ),
    nest(
        (path = "/pemonlist", api=pemonlist::ApiDoc),
        (path = "/", api=history::ApiDoc),
        (path = "/", api=queue::ApiDoc),
        (path = "/status", api=status::ApiDoc),
    ),
    components(
        schemas(
            Submission,
            SubmissionPage,
            SubmissionResolved,
            SubmissionStatus,
            Record,
            SubmissionPatchMod,
            SubmissionPatchUser,
            SubmissionInsert,
            SubmissionPage,
            SubmissionQueryOptions,
            SubmissionResolved,
            ResolvedSubmissionPage,
        )
    ),
    paths(
        find_all,
        find_one,
        find_me,
        claim,
        create,
        patch,
        delete,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .service(claim)
            .service(find_me)
            .configure(pemonlist::init_routes)
            .configure(status::init_routes)
            .configure(history::init_routes)
            .configure(queue::init_routes)
            .service(find_one)
            .service(patch)
            .service(delete)
            .service(create)
            .service(find_all),
    );
}
