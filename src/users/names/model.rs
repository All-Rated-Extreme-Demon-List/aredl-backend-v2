use std::collections::HashMap;
use diesel::{QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use itertools::Itertools;
use utoipa::ToSchema;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::users::{BaseUser, Role};
use crate::schema::{roles, user_roles, users};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name = user_roles, check_for_backend(Pg))]
pub struct UserRole {
    pub role_id: i32,
    pub user_id: Uuid,
}

#[derive(Serialize, Deserialize, Debug,ToSchema)]
pub struct RoleResolved {
    #[serde(flatten)]
    pub role: Role,
    /// Users with this role.
    pub users: Vec<BaseUser>,
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
            .select((user_roles::role_id, BaseUser::as_select()))
            .order_by(user_roles::role_id)
            .load::<(i32, BaseUser)>(conn)?;

        let result = user_roles
            .into_iter()
            .chunk_by(|(role_id, _user)| role_id.clone())
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