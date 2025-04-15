use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    auth::{UserAuth, Permission},
    aredl::submissions::{
        Submission, SubmissionInsert, SubmissionPatch
    },
    db::DbAppState, error_handler::ApiError
};

#[get("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(
        move || Submission::find_all(db)
    ).await??;
    Ok(HttpResponse::Ok().json(submissions))
}
#[get("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_one(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    tracing::debug!("{:?}", id.clone());
    let submission = web::block(
        move || Submission::find_one(db, id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[post("", wrap="UserAuth::load()")]
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsert>) -> Result<HttpResponse, ApiError> {
    let created = web::block(
        move || Submission::create(db, body.into_inner())
    ).await??;
    Ok(HttpResponse::Created().json(created))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn patch(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, body: web::Json<SubmissionPatch>) -> Result<HttpResponse, ApiError> {
    let patched = web::block(
        move || SubmissionPatch::patch(body.into_inner(), id.into_inner(), db)
    ).await??;
    Ok(HttpResponse::Ok().json(patched))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    web::block(
        move || Submission::delete(db, id.into_inner())
    ).await??;
    Ok(HttpResponse::NoContent().finish())
}

// todo
#[post("/{id}/claim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn claim(_db: web::Data<Arc<DbAppState>>, _id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::ImATeapot().finish())
}
#[post("/{id}/unclaim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn unclaim(_db: web::Data<Arc<DbAppState>>, _id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::ImATeapot().finish())
}
#[post("/{id}/accept", wrap="UserAuth::require(Permission::RecordModify)")]
async fn accept(_db: web::Data<Arc<DbAppState>>, _id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::ImATeapot().finish())
}
#[post("/{id}/deny", wrap="UserAuth::require(Permission::RecordModify)")]
async fn deny(_db: web::Data<Arc<DbAppState>>, _id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::ImATeapot().finish())
}
#[post("/{id}/underconsideration", wrap="UserAuth::require(Permission::RecordModify)")]
async fn under_consideration(_db: web::Data<Arc<DbAppState>>, _id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::ImATeapot().finish())
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .service(create)
            .service(find_all)
            .service(find_one)
            .service(patch)
            .service(delete)
            .service(claim)
            .service(unclaim)
            .service(accept)
            .service(deny)
            .service(under_consideration)
    );
}
