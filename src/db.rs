
use crate::error_handler::ApiError;
use std::env;
use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;
use lazy_static::lazy_static;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

//embed_migrations!();

lazy_static! {
    static ref POOL: Pool = {
        let db_url = env::var("DATABASE_URL").expect("Database url not set");
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Failed to create db pool")
    };
}

pub fn init() {
    lazy_static::initialize(&POOL);
    connection().expect("Failed to get db connection");
}

pub fn connection() -> Result<DbConnection, ApiError> {
    POOL.get().map_err(|e| ApiError::new(500, &format!("Failed to get db connection: {}", e)))
}