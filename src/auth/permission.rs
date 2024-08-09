use std::sync::Arc;
use actix_web::web;
use diesel::dsl::max;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use strum_macros::{EnumString, Display};
use uuid::Uuid;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::{permissions, roles, user_roles};

#[derive(Clone, EnumString, Display)]
#[strum(serialize_all="snake_case")]
pub enum Permission {
    LevelModify,
    RecordModify,
    PackTierModify,
    PackModify,
}

fn get_privilege_level(db: web::Data<Arc<DbAppState>>, user_id: Uuid) -> Result<i32, ApiError> {
    let privilege_level: Option<i32> = user_roles::table
        .inner_join(roles::table.on(roles::id.eq(user_roles::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .select(max(roles::privilege_level))
        .first(&mut db.connection()?)
        .unwrap_or(None);
    Ok(privilege_level.unwrap_or(0))
}

pub fn check_permission(db: web::Data<Arc<DbAppState>>, user_id: Uuid, permission: Permission) -> Result<bool, ApiError> {
    let max_privilege = get_privilege_level(db.clone(), user_id)?;
    let required_privilege = permissions::table
        .filter(permissions::permission.eq(permission.to_string()))
        .select(permissions::privilege_level)
        .first::<i32>(&mut db.connection()?)?;
    Ok(required_privilege <= max_privilege)
}