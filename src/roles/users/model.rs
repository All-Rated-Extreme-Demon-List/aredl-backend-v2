use std::sync::Arc;
use actix_web::web;
use diesel::{Connection, ExpressionMethods, insert_into, QueryDsl, RunQueryDsl, JoinOnDsl, SelectableHelper};
use uuid::Uuid;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::{users, user_roles};
use crate::users::BaseUser;


impl BaseUser {
    pub fn role_add_all(db: web::Data<Arc<DbAppState>>, role_id: i32, users: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::add_users(role_id, users.as_ref(), connection)?;

            let users: Vec<BaseUser> = user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
                .select(BaseUser::as_select())
                .load(connection)?;
            Ok(users)
        })
    }

    pub fn role_set_all(db: web::Data<Arc<DbAppState>>, role_id: i32, users: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            diesel::delete(user_roles::table.filter(user_roles::role_id.eq(role_id)))
                .execute(connection)?;

            Self::add_users(role_id, &users, connection)?;

			let users: Vec<BaseUser> = user_roles::table
				.filter(user_roles::role_id.eq(role_id))
				.inner_join(users::table.on(user_roles::user_id.eq(users::id)))
				.select(BaseUser::as_select())
				.load(connection)?;
			Ok(users)
        })
    }

    pub fn role_delete_all(db: web::Data<Arc<DbAppState>>, role_id: i32, users: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::delete_users(role_id, &users, connection)?;

			let users: Vec<BaseUser> = user_roles::table
				.filter(user_roles::role_id.eq(role_id))
				.inner_join(users::table.on(user_roles::user_id.eq(users::id)))
				.select(BaseUser::as_select())
				.load(connection)?;
			Ok(users)
        })
    }

    fn add_users(role_id: i32, users: &Vec<Uuid>, conn: &mut DbConnection) -> Result<(), ApiError> {
        insert_into(user_roles::table)
            .values(
                users.into_iter().map(|user| (
                    user_roles::user_id.eq(user),
                    user_roles::role_id.eq(role_id)
                )).collect::<Vec<_>>()
            )
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_users(role_id: i32, users: &Vec<Uuid>, conn: &mut DbConnection) -> Result<(), ApiError> {
            diesel::delete(
                user_roles::table
                    .filter(user_roles::role_id.eq(role_id))
                    .filter(user_roles::user_id.eq_any(users)),
            )
                .execute(conn)?;
            Ok(())
    }
}