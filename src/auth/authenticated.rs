use crate::app_data::db::DbConnection;
use crate::auth::token::UserClaims;
use crate::auth::{permission, Permission};
use crate::clans::ClanMember;
use crate::error_handler::ApiError;
use crate::schema::clan_members;
use crate::users::User;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage as _, HttpRequest};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

use diesel::prelude::*;
#[derive(Serialize, Deserialize, Debug)]
pub struct Authenticated(UserClaims);

impl FromRequest for Authenticated {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value = req.extensions().get::<UserClaims>().cloned();

        let result = match value {
            Some(claims) => Ok(Authenticated(claims)),
            None => Err(ApiError::Unauthorized("Authentication error")),
        };

        ready(result)
    }
}

impl Authenticated {
    pub fn ensure_not_banned(&self, conn: &mut DbConnection) -> Result<(), ApiError> {
        if User::is_banned(self.user_id, conn)? {
            return Err(ApiError::Forbidden("You have been banned from the list."));
        }
        Ok(())
    }

    pub fn ensure_has_permission(
        &self,
        conn: &mut DbConnection,
        permission: Permission,
    ) -> Result<(), ApiError> {
        if !self.has_permission(conn, permission)? {
            return Err(ApiError::Forbidden(format!(
                "You do not have the required permission ({permission}) to perform this action"
            )));
        }
        Ok(())
    }

    pub fn ensure_has_higher_privilege_than_user(
        &self,
        conn: &mut DbConnection,
        target_user_id: Uuid,
    ) -> Result<(), ApiError> {
        let acting_user_privilege =
            permission::get_highest_role_privilege_level(conn, self.user_id);
        let target_user_privilege =
            permission::get_highest_role_privilege_level(conn, target_user_id);

        if acting_user_privilege <= target_user_privilege {
            return Err(ApiError::Forbidden(
                "You do not have sufficient privilege to affect this user.",
            ));
        }

        Ok(())
    }

    pub fn has_higher_privilege_than(
        &self,
        conn: &mut DbConnection,
        required_privilege: i32,
    ) -> bool {
        let user_privilege = permission::get_highest_role_privilege_level(conn, self.user_id);
        user_privilege > required_privilege
    }

    pub fn has_permission(
        &self,
        conn: &mut DbConnection,
        permission: Permission,
    ) -> Result<bool, ApiError> {
        permission::check_user_permission(conn, self.user_id, permission)
    }

    pub fn get_permissions(&self, conn: &mut DbConnection) -> Result<Vec<String>, ApiError> {
        permission::get_user_permissions(conn, self.user_id, false)
    }

    pub fn ensure_has_clan_permission(
        &self,
        conn: &mut DbConnection,
        clan_id: Uuid,
        clan_role_level: i32,
    ) -> Result<(), ApiError> {
        let member = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(clan_members::user_id.eq(self.user_id))
            .select(ClanMember::as_select())
            .first::<ClanMember>(conn)
            .optional()?;

        let has_permission = self.has_permission(conn, Permission::ClanModify)?;
        if member.is_none_or(|member| member.role < clan_role_level) && !has_permission {
            return Err(ApiError::Forbidden(
                "You do not have the required permission to perform this action",
            ));
        }

        Ok(())
    }

    pub fn ensure_has_clan_higher_permission_than_user(
        &self,
        conn: &mut DbConnection,
        clan_id: Uuid,
        target_member_id: Uuid,
    ) -> Result<(), ApiError> {
        let member = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(clan_members::user_id.eq(target_member_id))
            .select(ClanMember::as_select())
            .first::<ClanMember>(conn)
            .optional()?;

        if let Some(member) = member {
            self.ensure_has_clan_permission(conn, clan_id, member.role)?;
        }

        Ok(())
    }
}

impl std::ops::Deref for Authenticated {
    type Target = UserClaims;

    /// Implement the deref method to access the inner User value of Authenticated.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
