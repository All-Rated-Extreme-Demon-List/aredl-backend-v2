use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;
use utoipa::OpenApi;
use crate::{
    aredl::{
        submissions::{
            RejectionData, 
            Submission, 
            SubmissionInsert, 
            SubmissionPage, 
            SubmissionPatch, 
            SubmissionQueryOptions, 
            SubmissionQueue, 
            SubmissionResolved, 
            SubmissionStatus
        }, 
        records::Record
    },
    auth::{Authenticated, Permission, UserAuth}, 
    db::DbAppState, 
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
};

#[utoipa::path(
    get,
    summary = "[Staff]List submissions",
    description = "Get a possibly filtered list of submissions.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = Paginated<SubmissionPage>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all(db: web::Data<Arc<DbAppState>>, page_query: web::Query<PageQuery<50>>, options: web::Query<SubmissionQueryOptions>) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(
        move || SubmissionPage::find_all(db, page_query.into_inner(), options.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    summary = "[Auth]Get a submission",
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
#[get("/{id}", wrap="UserAuth::load()")]
async fn find_one(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let submission = web::block(
        move || SubmissionResolved::find_one(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[derive(serde::Serialize, utoipa::ToSchema)]
pub struct QueuePositionResponse {
    pub position: i64,
    pub total: i64,
}

#[utoipa::path(
    get,
    summary = "[Auth]Get queue position for a submission",
    description = "Returns the position of a specific submission in the pending queue.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, description = "Queue position found", body = QueuePositionResponse),
        (status = 404, description = "Submission not found or not pending"),
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission to check position for")
    )
)]
#[get("/{id}/queue-position", wrap="UserAuth::load()")]
async fn get_queue_position(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    _auth: Authenticated
) -> Result<HttpResponse, ApiError> {
    let id = id.into_inner();

    let (position, total) = web::block(move || Submission::get_queue_position(db, id)).await??;

    Ok(HttpResponse::Ok().json(QueuePositionResponse { position, total }))
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
    )
)]
#[get("/@me", wrap="UserAuth::load()")]
async fn me(db: web::Data<Arc<DbAppState>>, page_query: web::Query<PageQuery<50>>, options: web::Query<SubmissionQueryOptions>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(
        move || SubmissionPage::find_own(db, page_query.into_inner(), options.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[utoipa::path(
    get,
    summary = "Get submissions queue",
    description = "Get the amount of pending submissions.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionQueue)
    )
)]
#[get("/queue")]
async fn get_queue(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let submission = web::block(
        move || SubmissionQueue::get_queue(db)
    ).await??;
    Ok(HttpResponse::Ok().json(submission))
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
    )
)]
#[post("", wrap="UserAuth::load()")]
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsert>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let created = web::block(
        move || Submission::create(db, body.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit a submission",
    description = "Edit a submission. If you aren't staff, the submission must be submitted by you and in the pending state.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = SubmissionPatch)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[patch("/{id}", wrap="UserAuth::load()")]
async fn patch(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, body: web::Json<SubmissionPatch>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    
    let mut conn = db.connection()?;
    let has_auth = authenticated.has_permission(db, Permission::RecordModify)?;
    
    let patched = web::block(
        move || SubmissionPatch::patch(body.into_inner(), id.into_inner(), &mut conn, has_auth, authenticated.user_id)
    ).await??;
    Ok(HttpResponse::Ok().json(patched))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete a submission",
    description = "Delete a submission by its ID. If you are staff, the submission must be yours and in the pending state.",
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
#[delete("/{id}", wrap="UserAuth::load()")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    web::block(
        move || Submission::delete(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::NoContent().finish())
}

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
#[get("/claim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn claim(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {

    let patched = web::block(
        move || SubmissionResolved::find_highest_priority(db, authenticated.user_id)
    ).await??;

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
#[post("/{id}/unclaim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn unclaim(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {

    let patched = web::block(
        move || Submission::unclaim(db, id.into_inner())
    ).await??;
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
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post("/{id}/accept", wrap="UserAuth::require(Permission::RecordModify)")]
async fn accept(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let new_record = web::block(
        move || Submission::accept(db, id.into_inner(), authenticated.user_id)
    ).await??;
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
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[post("/{id}/deny", wrap="UserAuth::require(Permission::RecordModify)")]
async fn deny(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated, body: Option<web::Json<RejectionData>>) -> Result<HttpResponse, ApiError> {

    let reason = match body {
        Some(body) => body.into_inner().reason,
        None => None
    };

    let new_record = web::block(
        move || Submission::reject(db, id.into_inner(), authenticated, reason)
    ).await??;
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
#[post("/{id}/underconsideration", wrap="UserAuth::require(Permission::RecordModify)")]
async fn under_consideration(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let new_record = web::block(
        move || Submission::under_consideration(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Ok().json(new_record))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Levels (Submissions)", description = "Endpoints for fetching and managing submissions")
    ),
    components(
        schemas(
            RejectionData, 
            Submission, 
            SubmissionInsert, 
            SubmissionPage, 
            SubmissionPatch, 
            SubmissionQueryOptions, 
            SubmissionQueue, 
            SubmissionResolved, 
            SubmissionStatus,
            Record
        )
    ),
    paths(
        create,
        find_all,
        me,
        get_queue,
        claim,
        find_one,
        get_queue_position,
        patch,
        delete,
        unclaim,
        accept,
        deny,
        under_consideration
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .service(create)
            .service(find_all)
            .service(me)
            .service(get_queue)
            .service(claim)
            .service(find_one)
            .service(get_queue_position)
            .service(patch)
            .service(delete)
            .service(unclaim)
            .service(accept)
            .service(deny)
            .service(under_consideration)
    );
}
