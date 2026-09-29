use crate::app_data::db::DbAppState;
use crate::auth::Authenticated;
use crate::auth::{Permission, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::page_helper::{PageQuery, Paginated};
use crate::users::merge::requests::{
    MergeRequest, MergeRequestPage, MergeRequestQueryOptions, MergeRequestUpsert,
    ResolvedMergeRequest,
};
use actix_web::web;
use actix_web::{get, post, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::{OpenApi, ToSchema};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MergeRequestOptions {
    /// The secondary user to merge, whose data will be merged into the authenticated user.
    pub secondary_user: Uuid,
}

#[utoipa::path(
    get,
    summary = "[Staff]Get merge request",
    description = "Get information about a specific merge request",
    tag = "Users - Merges",
    params(
		("id" = Uuid, Path, description = "Internal UUID of the merge request to find"),
	),
    responses(
        (status = 200, body = ResolvedMergeRequest),
        (status = 404, description = "Merge request not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[get("/{id}", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn find_one(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || ResolvedMergeRequest::find_one(&mut db.connection()?, id.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    get,
    summary = "[Staff]Get merge requests",
    description = "Paginated list of pending/denied merge requests",
    tag = "Users - Merges",
    params(
        ("claimed_filter" = Option<bool>, Query, description = "Filter merge requests to claimed/unclaimed requests"),
        ("rejected_filter" = Option<bool>, Query, description = "Filter merge requests to rejected/pending requests"),
        ("user_filter" = Option<String>, Query, description = "Filter merge requests involving a specific user (UUID, discord ID, or username)"),
		PageQuery<20>,
	),
    responses(
        (status = 200, body = Paginated<MergeRequestPage>),
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[get("", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<20>>,
    options: web::Query<MergeRequestQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        MergeRequestPage::find_all(
            &mut db.connection()?,
            page_query.into_inner(),
            &options.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Claim merge request",
    description = "Finds the oldest unclaimed merge request, marks it as claimed and returns it.",
    tag = "Users - Merges",
    responses(
        (status = 200, body = MergeRequest),
        (status = 204, description = "There are no merge requests available to claim"),
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[post("/claim", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn claim(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || MergeRequest::claim(&mut db.connection()?)).await??;
    Ok(match result {
        Some(item) => HttpResponse::Ok().json(item),
        None => HttpResponse::NoContent().finish(),
    })
}

#[utoipa::path(
    post,
    summary = "[Auth]Create merge request",
    description = "Creates a new merge request for the given user (secondary user) to be merged into the authenticated user (primary user).",
    tag = "Users - Merges",
    request_body = MergeRequestOptions,
    responses(
        (status = 200, body = MergeRequest),
        (status = 403, description = "You have been banned from the list", body = ErrorResponse),
        (status = 404, description = "Requesting or secondary user not found", body = ErrorResponse, examples(
            ("requesting_user" = (value = json!({"message": "Not found"}))),
            ("secondary_user" = (value = json!({"message": "The secondary user does not exist."})))
        )),
        (status = 409, description = "Secondary user is not a placeholder, or you already have a pending merge request", body = ErrorResponse, examples(
            ("not_placeholder" = (value = json!({"message": "You can only submit merge requests for placeholder users. To merge your account with a user that is already linked to another discord account, please make a support post on our discord server."}))),
            ("pending_request" = (value = json!({"message": "You already submitted a merge request for your account. Please wait until it's either accepted or denied before submitting a new one."})))
        )),
        (status = 422, description = "You cannot merge your account with itself", body = ErrorResponse)
    ),
    security(("bearer_token" = [])),
)]
#[post("", wrap = "UserAuth::load()")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<MergeRequestOptions>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&options));
    let result = web::block(move || {
        let conn = &mut db.connection()?;

        authenticated.ensure_not_banned(conn)?;

        MergeRequest::upsert(
            conn,
            &MergeRequestUpsert {
                primary_user: authenticated.user_id,
                secondary_user: options.secondary_user,
            },
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Accept merge request",
    description = "Accepts an existing merge request, merges both users and deletes the request.",
    tag = "Users - Merges",
    params(
		("id" = Uuid, Path, description = "Internal UUID of the merge request to accept"),
	),
    responses(
        (status = 204),
        (status = 404, description = "Merge request not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[post("/{id}/accept", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn accept(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || MergeRequest::accept(&mut db.connection()?, id.into_inner())).await??;

    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    summary = "[Staff]Deny merge request",
    description = "Rejects an existing merge request. Does not delete the request, but marks it as rejected.",
    tag = "Users - Merges",
    params(
		("id" = Uuid, Path, description = "Internal UUID of the merge request to reject"),
	),
    responses(
        (status = 200, body = MergeRequest),
        (status = 404, description = "Merge request not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[post("/{id}/reject", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn reject(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || MergeRequest::reject(&mut db.connection()?, id.into_inner())).await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Unclaim merge request",
    description = "Unclaims an existing merge request to make it available again",
    tag = "Users - Merges",
    params(
		("id" = Uuid, Path, description = "Internal UUID of the merge request to unclaim"),
	),
    responses(
        (status = 200, body = MergeRequest),
        (status = 404, description = "Merge request not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["MergeReview"])),
)]
#[post("/{id}/unclaim", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn unclaim(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || MergeRequest::unclaim(&mut db.connection()?, id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        ResolvedMergeRequest,
        MergeRequest,
        MergeRequestPage,
        MergeRequestOptions
    )),
    paths(list, find_one, claim, create, accept, reject, unclaim)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/requests")
            .service(list)
            .service(claim)
            .service(find_one)
            .service(create)
            .service(accept)
            .service(reject)
            .service(unclaim),
    );
}
