use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::roles::Role;
use crate::schema::{roles, user_roles, users};
use crate::users::BaseUser;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::ToSchema;

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct RoleResolved {
    #[serde(flatten)]
    pub role: Role,
    /// Users with this role.
    pub users: Vec<BaseUser>,
}

impl RoleResolved {
    pub fn find_all(conn: &mut DbConnection) -> Result<Vec<Self>, ApiError> {
        let roles: HashMap<i32, Role> = roles::table
            .filter(roles::hide.eq(false))
            .select(Role::as_select())
            .load::<Role>(conn)?
            .into_iter()
            .map(|role| (role.id, role))
            .collect();

        let user_roles = user_roles::table
            .inner_join(users::table)
            .select((user_roles::role_id, BaseUser::as_select()))
            .filter(user_roles::role_id.eq_any(roles.keys().copied().collect::<Vec<i32>>()))
            .order_by(user_roles::role_id)
            .load::<(i32, BaseUser)>(conn)?;

        let result = user_roles
            .into_iter()
            .chunk_by(|(role_id, _user)| role_id.clone())
            .into_iter()
            .map(|(role, users)| RoleResolved {
                role: Role {
                    id: role.clone(),
                    privilege_level: roles[&role].privilege_level,
                    role_desc: roles[&role].role_desc.clone(),
                    hide: roles[&role].hide,
                },
                users: users.map(|(_, users)| users).collect(),
            })
            .sorted_by_key(|v| -v.role.privilege_level)
            .collect_vec();

        Ok(result)
    }
}
