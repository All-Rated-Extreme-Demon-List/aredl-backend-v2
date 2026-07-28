#[cfg(test)]
use crate::auth::Permission;
use crate::error_handler::{ApiError, StartupError};
use crate::get_secret;
#[cfg(test)]
use crate::schema::permissions;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, PgConnection};
#[cfg(test)]
use diesel::{Connection as _, ExpressionMethods as _, RunQueryDsl as _};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness as _};
use std::sync::Arc;
#[cfg(test)]
use std::sync::Once;
#[cfg(test)]
use strum::IntoEnumIterator as _;

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
            .map_err(|e| ApiError::InternalServerError(format!("Failed to get db connection: {e}")))
    }

    pub fn run_pending_migrations(&self) -> Result<(), StartupError> {
        self.connection()?
            .run_pending_migrations(MIGRATIONS)
            .map_err(|error| {
                StartupError::Init(format!("Failed to run database migrations: {error}"))
            })?;

        Ok(())
    }
}

pub fn init_app_state() -> Result<Arc<DbAppState>, StartupError> {
    let db_url = format!(
        "postgres://{}:{}@db:5432/aredl",
        get_secret("POSTGRES_USER")?,
        get_secret("POSTGRES_PASSWORD")?
    );
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .map_err(|error| StartupError::Init(format!("Failed to start database pool: {error}")))?;

    Ok(Arc::new(DbAppState { pool }))
}

#[cfg(test)]
static INIT_DB: Once = Once::new();

#[cfg(test)]
fn init_test_db_schema_and_seed() {
    INIT_DB.call_once(|| {
        let test_db_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

        let mut conn = PgConnection::establish(&test_db_url).expect("Failed to connect to test DB");

        conn.revert_all_migrations(MIGRATIONS)
            .expect("Failed to revert migrations");

        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        diesel::insert_into(permissions::table)
            .values(
                Permission::iter()
                    .map(|permission| permissions::permission.eq(permission.to_string()))
                    .collect::<Vec<_>>(),
            )
            .execute(&mut conn)
            .expect("Failed to insert permissions");
    });
}

#[cfg(test)]
pub fn init_test_db_state() -> Arc<DbAppState> {
    use diesel::r2d2::TestCustomizer;

    init_test_db_schema_and_seed();

    let test_db_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set for running tests");

    let manager = ConnectionManager::<PgConnection>::new(test_db_url);
    let pool = r2d2::Pool::builder()
        .test_on_check_out(true)
        .max_size(1)
        .connection_customizer(Box::new(TestCustomizer))
        .build(manager)
        .expect("Failed to create test database pool");

    Arc::new(DbAppState { pool })
}
