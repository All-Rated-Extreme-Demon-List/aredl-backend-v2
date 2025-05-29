use crate::auth::Authenticated;
use crate::auth::{Permission, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::users::merge::requests::{
    MergeRequest, MergeRequestPage, MergeRequestUpsert, ResolvedMergeRequest,
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
        (status = 200, body = ResolvedMergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[get("/{id}", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn find_one(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        ResolvedMergeRequest::find_one(&mut conn, id.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    get,
    summary = "[Staff]Get merge requests",
    description = "Paginated list of pending/denied merge requests",
    tag = "Users - Merges",
    params(
		("page" = Option<i64>, Query, description = "The page of the merge requests to fetch"),
		("per_page" = Option<i64>, Query, description = "The number of merge requests to fetch per page"),
	),
    responses(
        (status = 200, body = Paginated<ResolvedMergeRequest>)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[get("", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<20>>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequestPage::find_all(&mut conn, page_query.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    get,
    summary = "[Staff]Claim merge request",
    description = "Finds the oldest unclaimed merge request, marks it as claimed and returns it.",
    tag = "Users - Merges",
    responses(
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[get("/claim", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn claim(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::claim(&mut conn)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Create merge request",
    description = "Creates a new merge request for the given user (secondary user) to be merged into the authenticated user (primary user).",
    tag = "Users - Merges",
    request_body = MergeRequestOptions,
    responses(
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("", wrap = "UserAuth::load()")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<MergeRequestOptions>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&options));
    let result = web::block(move || {
        let mut conn = db.connection()?;
        let merge_upsert = MergeRequestUpsert {
            primary_user: authenticated.user_id,
            secondary_user: options.secondary_user,
        };
        MergeRequest::upsert(&mut conn, merge_upsert)
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
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[post("/{id}/accept", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn accept(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::accept(&mut conn, id.into_inner())
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
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
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[post("/{id}/reject", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn reject(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::reject(&mut conn, id.into_inner())
    })
    .await??;

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
        (status = 200, body = MergeRequest)
    ),
    security(
        ("access_token" = ["MergeReview"]),
        ("api_key" = ["MergeReview"]),
    )
)]
#[post("/{id}/unclaim", wrap = "UserAuth::require(Permission::MergeReview)")]
async fn unclaim(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        MergeRequest::unclaim(&mut conn, id.into_inner())
    })
    .await??;
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
    paths(list, find_one, claim, create, accept, reject)
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
            .service(reject),
    );
}
