use actix_web::{delete, patch, post, web, HttpResponse};
use tracing_actix_web::RootSpan;
use std::sync::Arc;
use uuid::Uuid;
use utoipa::OpenApi;
use crate::{
    aredl::{
        records::Record, submissions::{
            actions, guidelines, patch::{SubmissionPatchMod, SubmissionPatchUser}, post::SubmissionInsert, status, Submission, SubmissionPage, SubmissionResolved, SubmissionStatus
        }
    },
    auth::{Authenticated, Permission, UserAuth}, 
    db::DbAppState, 
    error_handler::ApiError,
};

use super::{history, queue, resolved};


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
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsert>, authenticated: Authenticated, root_span: RootSpan) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&body));

    authenticated.check_is_banned(db.clone())?;

    let created = web::block(
        move || Submission::create(db, body.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Created().json(created))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit a submission",
    description = "Edit a submission. If you aren't staff, the submission must be yours and in the pending/denied state.",
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
)]
#[patch("/{id}", wrap="UserAuth::load()")]
async fn patch(
    db: web::Data<Arc<DbAppState>>, 
    id: web::Path<Uuid>, 
    body: web::Json<SubmissionPatchMod>, 
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&body));
    let has_auth = authenticated.has_permission(db.clone(), Permission::SubmissionReview)?;

    match has_auth {
        true => {
            let patched = web::block(
                move || SubmissionPatchMod::patch_mod(body.into_inner(), id.into_inner(), db, authenticated)
            ).await??;
            return Ok(HttpResponse::Ok().json(patched))
        }
        false => {
            let user_patch = SubmissionPatchMod::downgrade(body.into_inner());
            let patched = web::block(
                move || SubmissionPatchUser::patch(user_patch, id.into_inner(), db, authenticated)
            ).await??;
            return Ok(HttpResponse::Ok().json(patched))
        }
    }
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
#[delete("/{id}", wrap="UserAuth::load()")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    web::block(
        move || Submission::delete(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::NoContent().finish())
}



#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Submissions", description = "Endpoints for fetching and managing submissions")
    ),
    nest(
        (path = "/", api=actions::ApiDoc),
        (path = "/", api=history::ApiDoc),
        (path = "/", api=queue::ApiDoc),
        (path = "/", api=resolved::ApiDoc),
        (path = "/status", api=status::ApiDoc),
        (path = "/guidelines", api=guidelines::ApiDoc)
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
        create,
        patch,
        delete,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .configure(actions::init_routes)
            .configure(guidelines::init_routes)
            .configure(status::init_routes)
            .configure(history::init_routes)
            .configure(queue::init_routes)
            .configure(resolved::init_routes)
            .service(create)
            .service(patch)
            .service(delete)
    );
}
