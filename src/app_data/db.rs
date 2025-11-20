use crate::error_handler::ApiError;
use crate::get_secret;
use diesel::r2d2::ConnectionManager;
use diesel::{r2d2, PgConnection};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use std::sync::Arc;

#[cfg(test)]
use std::sync::Once;

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

#[cfg(test)]
static INIT_DB: Once = Once::new();

#[cfg(test)]
fn init_test_db_schema_and_seed() {
    INIT_DB.call_once(|| {
        use diesel::{Connection, RunQueryDsl};

        use crate::schema::permissions;

        let test_db_url =
            std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL must be set");

        let mut conn = PgConnection::establish(&test_db_url).expect("Failed to connect to test DB");

        conn.revert_all_migrations(MIGRATIONS)
            .expect("Failed to revert migrations");

        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let permissions_data = vec![
            ("plus", 5),
            ("submission_review", 15),
            ("record_modify", 20),
            ("placeholder_create", 25),
            ("user_modify", 25),
            ("pack_tier_modify", 30),
            ("pack_modify", 40),
            ("level_modify", 50),
            ("merge_review", 60),
            ("clan_modify", 70),
            ("notifications_subscribe", 75),
            ("user_ban", 85),
            ("direct_merge", 90),
            ("shift_manage", 95),
            ("role_manage", 100),
        ];

        diesel::insert_into(permissions::table)
            .values(
                permissions_data
                    .iter()
                    .map(|(permission, privilege_level)| {
                        use diesel::ExpressionMethods;

                        (
                            permissions::permission.eq(*permission),
                            permissions::privilege_level.eq(*privilege_level),
                        )
                    })
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

    let manager = ConnectionManager::<PgConnection>::new(test_db_url.clone());
    let pool = r2d2::Pool::builder()
        .test_on_check_out(true)
        .max_size(1)
        .connection_customizer(Box::new(TestCustomizer))
        .build(manager)
        .expect("Failed to create test database pool");

    Arc::new(DbAppState { pool })
}
