use crate::app_data::db::DbConnection;
use crate::auth::token::UserClaims;
use crate::auth::{permission, Permission};
use crate::clans::ClanMember;
use crate::error_handler::ApiError;
use crate::schema::clan_members;
use crate::users::User;
use actix_web::dev::Payload;
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use std::future::{ready, Ready};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Authenticated(UserClaims);

impl FromRequest for Authenticated {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value = req.extensions().get::<UserClaims>().cloned();

        let result = match value {
            Some(claims) => Ok(Authenticated(claims)),
            None => Err(ApiError::new(401, "Authentication error")),
        };

        ready(result)
    }
}

impl Authenticated {
    pub fn check_is_banned(&self, conn: &mut DbConnection) -> Result<(), ApiError> {
        if User::is_banned(self.user_id, conn)? {
            return Err(ApiError::new(
                403,
                "You have been banned from the list.".into(),
            ));
        }
        Ok(())
    }

    pub fn has_permission(
        &self,
        conn: &mut DbConnection,
        permission: Permission,
    ) -> Result<bool, ApiError> {
        permission::check_permission(conn, self.user_id, permission)
    }

    pub fn has_clan_permission(
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
        if (member.is_none() || member.unwrap().role < clan_role_level) && !has_permission {
            return Err(ApiError::new(
                403,
                "You do not have the required permission to perform this action".into(),
            ));
        }

        Ok(())
    }

    pub fn has_clan_higher_permission(
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

        if member.is_some() {
            self.has_clan_permission(conn, clan_id, member.unwrap().role)?;
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
