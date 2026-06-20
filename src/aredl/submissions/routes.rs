use crate::{
    app_data::db::DbAppState,
    aredl::{
        records::Record,
        submissions::{
            patch::{SubmissionPatchMod, SubmissionPatchUser},
            post::{SubmissionInsert, SubmissionPostMod},
            resolved::{ResolvedSubmissionPage, SubmissionQueryOptions},
            status, Submission, SubmissionPage, SubmissionResolved, SubmissionStatus,
        },
    },
    auth::{Authenticated, Permission, UserAuth},
    error_handler::ApiError,
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
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = Paginated<ResolvedSubmissionPage>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("level_filter" = Option<Uuid>, Query, description = "Filter submissions to a specific level UUID"),
        ("status_filter" = Option<SubmissionStatus>, Query, description = "Filter submissions to specific statuses"),
        ("mobile_filter" = Option<bool>, Query, description = "Filter submissions to mobile/desktop submissions only"),
        ("submitter_filter" = Option<String>, Query, description = "Filter submissions to a specific submitter (UUID, discord ID, or username)"),
        ("priority_filter" = Option<bool>, Query, description = "Filter submissions to priority/non-priority submissions"),
        ("reviewer_filter" = Option<String>, Query, description = "Filter submissions to a specific reviewer (UUID, discord ID, or username)"),
        ("note_filter" = Option<String>, Query, description = "Filter submissions that contain a specific note substring"),
))]
#[get("", wrap = "UserAuth::require(Permission::SubmissionReviewFull)")]
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
#[get("{id}", wrap = "UserAuth::load()")]
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
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = Paginated<SubmissionPage>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("page" = Option<i64>, Query, description = "The page of the list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of entries to fetch per page"),
        ("level_filter" = Option<Uuid>, Query, description = "Filter submissions to a specific level UUID"),
        ("status_filter" = Option<SubmissionStatus>, Query, description = "Filter submissions to specific statuses"),
        ("mobile_filter" = Option<bool>, Query, description = "Filter submissions to mobile/desktop submissions only")
))]
#[get("@me", wrap = "UserAuth::load()")]
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
    description = "Create a submission to be checked by a moderator.",
    tag = "AREDL - Submissions",
    responses(
        (status = 201, body = Submission)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    request_body = SubmissionPostMod,
)]
#[post("", wrap = "UserAuth::load()")]
async fn create(
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
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = Submission)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
    request_body = SubmissionPatchMod,
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
        if authenticated.has_permission(conn, Permission::SubmissionReviewBase)? {
            SubmissionPatchMod::patch(
                body.into_inner(),
                id.into_inner(),
                conn,
                &authenticated,
                notify_tx.get_ref(),
                providers.get_ref(),
            )
        } else {
            let user_patch = SubmissionPatchMod::downgrade(body.into_inner());
            SubmissionPatchUser::patch(
                user_patch,
                id.into_inner(),
                conn,
                &authenticated,
                providers.get_ref(),
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
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("/claim", wrap = "UserAuth::require(Permission::SubmissionReviewBase)")]
async fn claim(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let claimed = web::block(move || {
        Submission::claim_highest_priority(&mut db.connection()?, &authenticated)
    })
    .await??;

    Ok(HttpResponse::Ok().json(claimed))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete a submission",
    description = "Delete a submission by its ID. If you aren't staff, the submission must be yours and in the pending state.",
    tag = "AREDL - Submissions",
    responses(
        (status = 204)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
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
        (name = "AREDL - Submissions", description = "Endpoints for fetching and managing submissions")
    ),
    nest(
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
