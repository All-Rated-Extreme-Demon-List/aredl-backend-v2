use crate::list::submissions::sync;
use crate::list::List;
use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    list::{
        records::Record,
        submissions::{
            patch::{SubmissionPatchMod, SubmissionPatchUser},
            post::{SubmissionInsert, SubmissionPostMod},
            resolved::{ResolvedSubmissionPage, SubmissionQueryOptions, SubmissionsSortField},
            status, Submission, SubmissionPage, SubmissionResolved, SubmissionStatus,
        },
    },
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
    tag = "List - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>),
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("sort" = Option<SubmissionsSortField>, Query, description = "The sorting type to use. Sorting by completion time is only possible for platformer"),
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
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_all(
            &mut db.connection()?,
            *list,
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
    tag = "List - Submissions",
    responses(
        (status = 200, body = SubmissionResolved),
        (status = 404, description = "Submission not found or not visible to this user", body = ErrorResponse)
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, description = "The ID of the submission")
    ),
    security(("bearer_token" = [])),
)]
#[get("/{id}", wrap = "UserAuth::load()")]
async fn find_one(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<SubmissionPath>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submission = web::block(move || {
        SubmissionResolved::find_one(&mut db.connection()?, path.list, path.id, &authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get own submissions",
    description = "List all submissions submitted by the logged in user.",
    tag = "List - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>),
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("sort" = Option<SubmissionsSortField>, Query, description = "The sorting type to use. Completion-time sorting is only available for platformer."),
        ("priority_filter" = Option<bool>, Query, description = "Filter submissions to priority/non-priority submissions"),
        ("note_filter" = Option<String>, Query, description = "Filter submissions that contain a specific note substring"),
        PageQuery<50>,
        ("level_filter" = Option<Uuid>, Query, description = "Filter submissions to a specific level UUID"),
        ("status_filter" = Option<SubmissionStatus>, Query, description = "Filter submissions to specific statuses"),
        ("mobile_filter" = Option<bool>, Query, description = "Filter submissions to mobile/desktop submissions only")
),
    security(("bearer_token" = [])),
)]
#[get("/@me", wrap = "UserAuth::load()")]
async fn find_me(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<SubmissionQueryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(move || {
        ResolvedSubmissionPage::find_own(
            &mut db.connection()?,
            *list,
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
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 201, body = Submission),
        (status = 400, description = "Invalid completion video or raw footage URL", body = ErrorResponse, examples(
            ("invalid_video_url" = (value = json!({"message": "Invalid completion video URL: Malformed URL"}))),
            ("invalid_raw_url" = (value = json!({"message": "Invalid raw footage URL: Malformed URL"})))
        )),
        (status = 403, description = "You have been banned from the list, or submissions are currently disabled", body = ErrorResponse, examples(
            ("banned" = (value = json!({"message": "You have been banned from the list."}))),
            ("disabled" = (value = json!({"message": "Submissions are currently disabled"})))
        )),
        (status = 404, description = "User or level not found", body = ErrorResponse),
        (status = 409, description = "You already have a submission for this level", body = ErrorResponse),
        (status = 410, description = "This level has been removed from the list", body = ErrorResponse),
        (status = 422, description = "This level is on the legacy list, required raw footage is missing, the completion time does not match the selected list, or the completion video provider is unsupported or not allowed", body = ErrorResponse, examples(
            ("legacy_level" = (value = json!({"message": "This level is on the legacy list and is not accepting records."}))),
            ("raw_footage_required" = (value = json!({"message": "This level requires raw footage"}))),
            ("unsupported_video_url" = (value = json!({"message": "Invalid completion video URL: URL does not match any known supported providers. Please refer to our guidelines for a list of supported websites."}))),
            ("disallowed_video_provider" = (value = json!({"message": "Invalid completion video URL: This provider is not allowed for this field"})))
        ))
    ),
    request_body = SubmissionPostMod,
    security(("bearer_token" = [])),
)]
#[post("", wrap = "UserAuth::load()")]
async fn create(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<SubmissionPostMod>,
    authenticated: Authenticated,
    providers: web::Data<Arc<ProvidersAppState>>,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let created = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_not_banned(conn)?;
        Submission::create(
            conn,
            *list,
            body.into_inner(),
            &authenticated,
            providers.get_ref(),
            notify_tx.get_ref(),
        )
    })
    .await??;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit a submission",
    description = "Edit a submission. If you aren't staff, the submission must be yours and not being actively reviewed.",
    tag = "List - Submissions",
    responses(
        (status = 200, body = Submission),
        (status = 400, description = "Invalid completion video or raw footage URL, no changes provided, or editing a non-pending submission while submissions are closed", body = ErrorResponse, examples(
            ("invalid_video_url" = (value = json!({"message": "Invalid completion video URL: Malformed URL"}))),
            ("invalid_raw_url" = (value = json!({"message": "Invalid raw footage URL: Malformed URL"}))),
            ("no_changes" = (value = json!({"message": "No changes were provided!"}))),
            ("submissions_closed" = (value = json!({"message": "Submissions are currently closed. You can only edit pending submissions."})))
        )),
        (status = 403, description = "User is banned, submission is locked or belongs to another user, or reviewer permissions are insufficient", body = ErrorResponse, examples(
            ("banned" = (value = json!({"message": "You have been banned from submitting records."}))),
            ("not_submitter" = (value = json!({"message": "You can only edit your own submissions."}))),
            ("locked" = (value = json!({"message": "This submission has been locked and cannot be edited"}))),
            ("reviewer_permissions" = (value = json!({"message": "You do not have permission to edit this submission."})))
        )),
        (status = 404, description = "User, submission or level not found", body = ErrorResponse),
        (status = 409, description = "This submission is currently being reviewed and cannot be edited", body = ErrorResponse),
        (status = 410, description = "This level has been removed from the list", body = ErrorResponse),
        (status = 422, description = "Required raw footage is missing, the completion time does not match the selected list, or the completion video provider is unsupported or not allowed", body = ErrorResponse, examples(
            ("required_raw_footage" = (value = json!({"message": "This level requires raw footage"}))),
            ("platformer_raw_footage" = (value = json!({"message": "Platformer submissions require raw footage"}))),
            ("classic_completion_time" = (value = json!({"message": "Classic submissions cannot have a completion time"}))),
            ("platformer_completion_time" = (value = json!({"message": "Platformer submissions require a completion time"}))),
            ("unsupported_video_url" = (value = json!({"message": "Invalid completion video URL: URL does not match any known supported providers. Please refer to our guidelines for a list of supported websites."}))),
            ("disallowed_video_provider" = (value = json!({"message": "Invalid completion video URL: This provider is not allowed for this field"})))
        ))
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, description = "The ID of the submission")
    ),
    request_body = SubmissionPatchMod,
    security(("bearer_token" = [])),
)]
#[patch("/{id}", wrap = "UserAuth::load()")]
async fn patch(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<SubmissionPath>,
    body: web::Json<SubmissionPatchMod>,
    authenticated: Authenticated,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&body));
    let list = path.list;
    let db_clone = db.clone();
    let providers_clone = providers.clone();
    let patched = web::block(move || {
        let conn = &mut db.connection()?;
        if authenticated.has_permission(conn, Permission::SubmissionReview)? {
            SubmissionPatchMod::patch(
                body.into_inner(),
                path.id,
                conn,
                path.list,
                &authenticated,
                notify_tx.get_ref(),
                providers.get_ref(),
            )
        } else {
            let user_patch = SubmissionPatchMod::downgrade(body.into_inner());
            SubmissionPatchUser::patch(
                user_patch,
                path.id,
                conn,
                path.list,
                &authenticated,
                providers.get_ref(),
            )
        }
    })
    .await??;

    // if the status submission is changed to accepted, trigger other actions (timestamp update, badges, bounties, etc)
    if patched.status == SubmissionStatus::Accepted {
        Record::post_accept_actions(
            db_clone,
            list,
            std::slice::from_ref(&patched),
            providers_clone,
        );
    }
    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    post,
    summary = "[Staff]Claim a submission",
    description = "Claim the next submission to be checked. Alternates between priority and non-priority submissions when possible.",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 200, body = SubmissionResolved),
        (status = 204, description = "There are no submissions available to claim")
    ),
    security(("bearer_token" = ["SubmissionReview"])),
)]
#[post("/claim", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn claim(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let claimed = web::block(move || {
        Submission::claim_highest_priority(&mut db.connection()?, *list, &authenticated)
    })
    .await??;

    Ok(match claimed {
        Some(item) => HttpResponse::Ok().json(item),
        None => HttpResponse::NoContent().finish(),
    })
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete a submission",
    description = "Delete a submission by its ID. If you aren't staff, the submission must be yours and in the pending state.",
    tag = "List - Submissions",
    responses(
        (status = 204),
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, description = "The ID of the submission")
    ),
    security(("bearer_token" = [])),
)]
#[delete("/{id}", wrap = "UserAuth::load()")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<SubmissionPath>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        Submission::delete(&mut db.connection()?, path.list, path.id, &authenticated)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Submissions", description = "Endpoints for fetching and managing submissions")
    ),
    nest(
        (path = "/", api=history::ApiDoc),
        (path = "/", api=queue::ApiDoc),
        (path = "/status", api=status::ApiDoc),
        (path = "/sync", api=sync::ApiDoc)
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
            .configure(status::init_routes)
            .configure(history::init_routes)
            .configure(queue::init_routes)
            .configure(sync::init_routes)
            .service(find_one)
            .service(patch)
            .service(delete)
            .service(create)
            .service(find_all),
    );
}

#[derive(serde::Deserialize)]
pub struct SubmissionPath {
    pub list: List,
    pub id: Uuid,
}
