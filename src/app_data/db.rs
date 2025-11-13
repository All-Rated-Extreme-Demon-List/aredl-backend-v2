use crate::error_handler::ApiError;
use crate::get_secret;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use std::sync::Arc;

type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
pub type DbConnection = r2d2::PooledConnection<ConnectionManager<PgConnection>>;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct DbAppState {
    pub pool: Pool,
}

impl DbAppState {
    pub fn connection(&self) -> Result<DbConnection, ApiError> {
        self.pool
            .get()
            .map_err(|e| ApiError::new(500, &format!("Failed to get db connection: {}", e)))
    }

    pub fn run_pending_migrations(&self) {
        self.connection()
            .unwrap()
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run pending migrations!");
    }
}

pub fn init_app_state() -> Arc<DbAppState> {
    let db_url = format!(
        "postgres://{}:{}@db:5432/aredl",
        get_secret("POSTGRES_USER"),
        get_secret("POSTGRES_PASSWORD")
    );
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create db pool");

    Arc::new(DbAppState { pool })
}
