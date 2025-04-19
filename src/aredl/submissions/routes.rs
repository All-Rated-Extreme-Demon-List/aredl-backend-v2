use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    aredl::submissions::{
        RejectionData, Submission, SubmissionInsert, SubmissionPatch, SubmissionResolved, SubmissionStatus
    }, 
    auth::{Authenticated, Permission, UserAuth}, 
    db::DbAppState, error_handler::ApiError
};
use is_url::is_url;

#[get("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(
        move || Submission::find_all(db)
    ).await??;
    Ok(HttpResponse::Ok().json(submissions))
}
#[get("/{id}", wrap="UserAuth::load()")]
async fn find_one(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let submission = web::block(
        move || SubmissionResolved::find_one(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Ok().json(submission))
}

#[post("", wrap="UserAuth::load()")]
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsert>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let submission = body.into_inner();

    if !is_url(&submission.video_url) {
        return Err(ApiError::new(400, "Invalid completion URL"));
    }

    if let Some(raw_url) = submission.raw_url.as_ref() {
        if !is_url(raw_url) {
            return Err(ApiError::new(400, "Invalid raw footage URL"));
        }
    }
    
    let created = web::block(
        move || Submission::create(db, submission, authenticated)
    ).await??;
    Ok(HttpResponse::Created().json(created))
}

#[patch("/{id}", wrap="UserAuth::load()")]
async fn patch(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, body: web::Json<SubmissionPatch>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    
    let mut conn = db.connection()?;
    let has_auth = authenticated.has_permission(db, Permission::RecordModify)?;
    
    let patched = web::block(
        move || SubmissionPatch::patch(body.into_inner(), id.into_inner(), &mut conn, has_auth, authenticated.user_id)
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


#[post("/claim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn claim(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {

    let patched = web::block(
        move || SubmissionResolved::find_highest_priority(db, authenticated.user_id)
    ).await??;

    Ok(HttpResponse::Ok().json(patched))
}
#[post("/{id}/unclaim", wrap="UserAuth::require(Permission::RecordModify)")]
async fn unclaim(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let mut conn = db.connection()?;
    let new_data = SubmissionPatch {
        status: Some(SubmissionStatus::Pending),
        ..Default::default()
    };

    let new_record = web::block(
        move || SubmissionPatch::patch(new_data, id.into_inner(), &mut conn, true, authenticated.user_id)
    ).await??;
    let resolved = SubmissionResolved::from(new_record, db, None)?;
    Ok(HttpResponse::Ok().json(resolved))
}
#[post("/{id}/accept", wrap="UserAuth::require(Permission::RecordModify)")]
async fn accept(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let new_record = web::block(
        move || Submission::accept(db, id.into_inner(), authenticated.user_id)
    ).await??;
    Ok(HttpResponse::Accepted().json(new_record))
}
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
#[post("/{id}/underconsideration", wrap="UserAuth::require(Permission::RecordModify)")]
async fn under_consideration(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let new_record = web::block(
        move || Submission::under_consideration(db, id.into_inner(), authenticated)
    ).await??;
    Ok(HttpResponse::Ok().json(new_record))
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
