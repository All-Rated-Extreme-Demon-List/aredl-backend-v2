use crate::app_data::db::DbConnection;
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::roles::Role;
use crate::schema::{user_roles, users};
use crate::users::BaseUser;
use diesel::{
    insert_into, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};
use uuid::Uuid;

impl BaseUser {
    pub fn role_add_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            Role::user_can_edit(connection, authenticated, role_id)?;
            Self::add_users(role_id, users.as_ref(), connection)?;

            let users: Vec<BaseUser> = user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .inner_join(users::table.on(user_roles::user_id.eq(users::id)))
                .select(BaseUser::as_select())
                .load(connection)?;
            Ok(users)
        })
    }

    pub fn role_set_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            Role::user_can_edit(connection, authenticated, role_id)?;
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

    pub fn role_delete_all(
        conn: &mut DbConnection,
        role_id: i32,
        authenticated: Authenticated,
        users: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            Role::user_can_edit(connection, authenticated, role_id)?;
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
                users
                    .into_iter()
                    .map(|user| {
                        (
                            user_roles::user_id.eq(user),
                            user_roles::role_id.eq(role_id),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_users(
        role_id: i32,
        users: &Vec<Uuid>,
        conn: &mut DbConnection,
    ) -> Result<(), ApiError> {
        diesel::delete(
            user_roles::table
                .filter(user_roles::role_id.eq(role_id))
                .filter(user_roles::user_id.eq_any(users)),
        )
        .execute(conn)?;
        Ok(())
    }
}
