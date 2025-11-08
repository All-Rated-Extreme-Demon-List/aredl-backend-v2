use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::roles;
use diesel::{ExpressionMethods, RunQueryDsl};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug, ToSchema,
)]
#[diesel(table_name = roles)]
pub struct Role {
    /// Internal ID of the role.
    pub id: i32,
    /// Privilege level of the role. Refer to [API Overview](#overview) for more information.
    pub privilege_level: i32,
    /// Name of the role.
    pub role_desc: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=roles, check_for_backend(Pg))]
pub struct RoleCreate {
    /// Privilege level of the role to create.
    pub privilege_level: i32,
    /// Name of the role to create.
    pub role_desc: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=roles, check_for_backend(Pg))]
pub struct RoleUpdate {
    /// New privilege level of the role.
    pub privilege_level: Option<i32>,
    /// New name of the role.
    pub role_desc: Option<String>,
}

impl Role {
    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        let roles = roles::table.load(conn)?;
        Ok(roles)
    }

    pub fn create(conn: &mut DbConnection, role: RoleCreate) -> Result<Self, ApiError> {
        let role = diesel::insert_into(roles::table)
            .values(role)
            .get_result(conn)?;
        Ok(role)
    }

    pub fn update(conn: &mut DbConnection, id: i32, role: RoleUpdate) -> Result<Self, ApiError> {
        let role = diesel::update(roles::table)
            .filter(roles::id.eq(id))
            .set(role)
            .get_result(conn)?;
        Ok(role)
    }

    pub fn delete(conn: &mut DbConnection, id: i32) -> Result<Self, ApiError> {
        let role = diesel::delete(roles::table)
            .filter(roles::id.eq(id))
            .get_result(conn)?;
        Ok(role)
    }
}
