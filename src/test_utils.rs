use std::sync::{Once, Arc};
use uuid::Uuid;
use actix_web::{test, App};
use actix_web::web::Data;
use diesel::{PgConnection, RunQueryDsl, ExpressionMethods, QueryDsl};
use diesel::r2d2::{self, ConnectionManager};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use crate::auth::{init_app_state, AuthAppState, Permission};
use crate::db::{DbAppState, DbConnection};
use crate::schema::permissions;
use crate::schema::{users, roles, user_roles};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();
static INIT: Once = Once::new();

pub fn init_test_db_state() -> Arc<DbAppState> {

    let test_db_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set for running tests");

    let manager = ConnectionManager::<PgConnection>::new(test_db_url);
    let pool = r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create test database pool");

    let test_db_state = Arc::new(DbAppState { pool });

    INIT.call_once(|| {

        test_db_state.connection().unwrap().revert_all_migrations(MIGRATIONS)
            .expect("Failed to revert test migrations");
        test_db_state.run_pending_migrations();

    let mut conn = test_db_state.connection().unwrap();

    let permissions_data = vec![
            ("record_modify", 20),
            ("placeholder_create", 25),
            ("user_modify", 25),
            ("pack_tier_modify", 30),
            ("pack_modify", 40),
            ("level_modify", 50),
            ("merge_review", 60),
            ("clan_modify", 70),
            ("user_ban", 85),
            ("direct_merge", 90),
            ("role_manage", 100)
        ];

    diesel::insert_into(permissions::table)
        .values(
            permissions_data
                .iter()
                .map(|(permission, privilege_level)| {
                    (
                        permissions::permission.eq(*permission),
                        permissions::privilege_level.eq(*privilege_level),
                    )
                })
                .collect::<Vec<_>>(),
        ).execute(&mut conn).expect("Failed to insert permissions");

    });

    test_db_state
}

#[cfg(test)]
pub async fn init_test_app() -> (impl actix_web::dev::Service<
    actix_http::Request, 
    Response = actix_web::dev::ServiceResponse, 
    Error = actix_web::Error,
>,  DbConnection, Arc<AuthAppState>) {

    dotenv::dotenv().ok();

    let auth_app_state= init_app_state().await;

    let db_app_state = init_test_db_state();
    let conn = db_app_state.connection().unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(db_app_state))
            .app_data(Data::new(auth_app_state.clone()))
            .configure(crate::users::init_routes)
    )
    .await;

    (app, conn, auth_app_state)
}

#[cfg(test)]
pub async fn create_test_user(conn: &mut DbConnection, required_permission: Option<Permission>) -> (Uuid, String) {
    let user_id = Uuid::new_v4();
    let username = format!("test_user_{}", user_id); 

    diesel::insert_into(users::table)
        .values((
            users::id.eq(user_id),
            users::username.eq(&username),
            users::global_name.eq(&username),
            users::discord_id.eq(None::<String>),
            users::placeholder.eq(false),
            users::country.eq(None::<i32>),
            users::discord_avatar.eq(None::<String>),
            users::discord_banner.eq(None::<String>),
            users::discord_accent_color.eq(None::<i32>),
        ))
        .execute(conn)
        .expect("Failed to create fake user");

    if required_permission.is_some() {

        let privilege_level = permissions::table
            .filter(permissions::permission.eq(required_permission.unwrap().to_string()))
            .select(permissions::privilege_level)
            .first::<i32>(conn)
            .expect("Failed to get privilege level");

        let role_id: i32 = diesel::insert_into(roles::table)
            .values((
                roles::privilege_level.eq(privilege_level),
                roles::role_desc.eq(format!("Test Role - {}", privilege_level)),
            ))
            .returning(roles::id)
            .get_result(conn)
            .expect("Failed to create test role");

        diesel::insert_into(user_roles::table)
            .values((
                user_roles::role_id.eq(role_id),
                user_roles::user_id.eq(user_id),
            ))
            .execute(conn)
            .expect("Failed to assign role to user");
        }

    (user_id, username)
}