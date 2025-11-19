use actix_web::{HttpResponse, delete, get, patch, post, web};
use tracing_actix_web::RootSpan;
use std::sync::Arc;
use uuid::Uuid;
use utoipa::OpenApi;
use crate::{
    app_data::db::DbAppState, arepl::{
        records::Record, submissions::{
             Submission, SubmissionPage, SubmissionResolved, SubmissionStatus, patch::{SubmissionPatchMod, SubmissionPatchUser}, pemonlist, post::{SubmissionInsert, SubmissionInsertBody}, statistics, status
        }
    }, auth::{Authenticated, Permission, UserAuth}, error_handler::ApiError, notifications::WebsocketNotification
};
use tokio::sync::broadcast;

use super::{history, queue, resolved};


#[utoipa::path(
    post,
    summary = "[Auth]Create a submission",
    description = "Create a submission to be checked by a moderator.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 201, body = Submission)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    request_body = SubmissionInsertBody,
)]
#[post("", wrap="UserAuth::load()")]
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsertBody>, authenticated: Authenticated, root_span: RootSpan) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&body));
    let created = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.check_is_banned(conn)?;
        Submission::create(conn, body.into_inner(), authenticated)
    }).await??;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit a submission",
    description = "Edit a submission. If you aren't staff, the submission must be yours and not being actively reviewed.",
    tag = "AREDL (P) - Submissions",
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
)]
#[patch("/{id}", wrap="UserAuth::load()")]
async fn patch(
    db: web::Data<Arc<DbAppState>>, 
    id: web::Path<Uuid>, 
    body: web::Json<SubmissionPatchMod>, 
    authenticated: Authenticated,
    root_span: RootSpan,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&body));
    let patched = web::block(move || {
        let conn = &mut db.connection()?;
        match authenticated.has_permission( conn, Permission::SubmissionReview)? {
            true => SubmissionPatchMod::patch(body.into_inner(), id.into_inner(), conn, authenticated, notify_tx.get_ref().clone()),
            false => {
                let user_patch = SubmissionPatchMod::downgrade(body.into_inner());
                SubmissionPatchUser::patch(user_patch, id.into_inner(), conn, authenticated)
            }
        }
    }).await??;
    Ok(HttpResponse::Ok().json(patched))
}


#[utoipa::path(
    get,
    summary = "[Staff]Claim a submission",
    description = "Claim the submission with the highest priority to be checked.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = SubmissionResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("/claim", wrap = "UserAuth::require(Permission::SubmissionReview)")]
async fn claim(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let patched = web::block(move || {
        Submission::claim_highest_priority(&mut db.connection()?, authenticated)
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
        move || Submission::delete(&mut db.connection()?, id.into_inner(), authenticated)
    ).await??;
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
        (path = "/", api=resolved::ApiDoc),
        (path = "/status", api=status::ApiDoc),
        (path = "/statistics", api=statistics::ApiDoc)
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
        )
    ),
    paths(
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
            .configure(pemonlist::init_routes)
            .configure(statistics::init_routes)
            .configure(status::init_routes)
            .configure(history::init_routes)
            .configure(queue::init_routes)
            .configure(resolved::init_routes)
            .service(create)
            .service(patch)
            .service(delete)
            
    );
}
