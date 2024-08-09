use crate::error_handler::ApiError;
use std::env;
use std::sync::Arc;
use diesel::{PgConnection, r2d2};
use diesel::r2d2::ConnectionManager;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct DbAppState {
    pub pool: Pool
}

impl DbAppState {
    pub fn connection(&self) -> Result<DbConnection, ApiError> {
        self.pool.get()
            .map_err(|e| ApiError::new(500, &format!("Failed to get db connection: {}", e)))
    }

    pub fn run_pending_migrations(&self) {
        self.connection().unwrap().run_pending_migrations(MIGRATIONS)
            .expect("Failed to run pending migrations!");
    }
}

pub fn init_app_state() -> Arc<DbAppState> {
    let db_url = env::var("DATABASE_URL").expect("Database url not set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create db pool");

    Arc::new(DbAppState { pool })
}
