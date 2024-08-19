use std::collections::HashMap;
use diesel::{BelongingToDsl, GroupedBy, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use diesel::query_dsl::methods::OrderDsl;
use itertools::Itertools;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{roles, user_roles, users};

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug)]
#[diesel(table_name = roles)]
pub struct Role {
    pub id: i32,
    pub privilege_level: i32,
    pub role_desc: String,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, PartialEq, Debug)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub global_name: String,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name = user_roles, check_for_backend(Pg))]
pub struct UserRole {
    pub role_id: i32,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RoleResolved {
    #[serde(flatten)]
    pub role: Role,
    pub users: Vec<User>,
}

impl RoleResolved {
    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        let roles: HashMap<i32, Role> = roles::table
            .select(Role::as_select())
            .load::<Role>(conn)?
            .into_iter()
            .map(|role| (role.id, role))
            .collect();

        let user_roles = user_roles::table
            .inner_join(users::table)
            .select((user_roles::role_id, User::as_select()))
            .load::<(i32, User)>(conn)?;

        let result = user_roles
            .into_iter()
            .chunk_by(|(role_id, user)| role_id.clone())
            .into_iter()
            .map(|(role, users)|
                RoleResolved {
                    role: Role {
                        id: role.clone(),
                        privilege_level: roles[&role].privilege_level,
                        role_desc: roles[&role].role_desc.clone(),
                    },
                    users: users.map(|(_, users)| users).collect(),
                }
            )
            .sorted_by_key(|v| -v.role.privilege_level)
            .collect_vec();

        Ok(result)
    }
}