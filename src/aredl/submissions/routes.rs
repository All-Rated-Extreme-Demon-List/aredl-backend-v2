use actix_web::{post, delete, get, web, HttpResponse};
use std::sync::Arc;
use uuid::Uuid;
use crate::{
    auth::{UserAuth, Permission},
    aredl::submissions::{
            Submission, SubmissionInsert
        },
    db::DbAppState, error_handler::ApiError
};

#[post("")]
async fn create(db: web::Data<Arc<DbAppState>>, body: web::Json<SubmissionInsert>) -> Result<HttpResponse, ApiError> {
    // todo: resolve level ID
    let created = web::block(
        move || Submission::create(db, body.into_inner())
    ).await??;
    Ok(HttpResponse::Created().json(created))
}

#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let submissions = web::block(
        move || Submission::find_all(db)
    ).await??;
    Ok(HttpResponse::Ok().json(submissions))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let deleted = web::block(
        move || Submission::delete(db, id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(deleted))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .service(create)
            .service(find_all)
            .service(delete)
    );
}
