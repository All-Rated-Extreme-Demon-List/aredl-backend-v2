use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use crate::aredl::leaderboard::model::{LeaderboardPage, LeaderboardQueryOptions};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::PageQuery;

#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<LeaderboardQueryOptions>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        LeaderboardPage::find(&mut conn, page_query.into_inner(), options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/leaderboard")
            .service(list)
    );
}