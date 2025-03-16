use std::future::{Ready, ready};
use std::sync::Arc;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use actix_web::dev::Payload;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::{QueryDsl, ExpressionMethods, OptionalExtension, RunQueryDsl, SelectableHelper};
use crate::auth::{Permission, permission};
use crate::auth::token::UserClaims;
use crate::clans::ClanMember;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::clan_members;

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
    pub fn has_permission(&self, db: web::Data<Arc<DbAppState>>, permission: Permission) -> Result<bool, ApiError> {
        permission::check_permission(db, self.user_id, permission)
    }

    pub fn has_clan_permission(&self, db: web::Data<Arc<DbAppState>>, clan_id: Uuid, clan_role_level: i32) -> Result<(), ApiError> {
        let member = clan_members::table
			.filter(clan_members::clan_id.eq(clan_id))
			.filter(clan_members::user_id.eq(self.user_id))
			.select(ClanMember::as_select())
			.first::<ClanMember>(&mut db.connection()?)
			.optional()?;

		let has_permission = self.has_permission(db, Permission::ClanModify)?;
		if (member.is_none() || member.unwrap().role < clan_role_level ) && !has_permission {
			return Err(ApiError::new(403, "You do not have the required permission to perform this action".into()));
		}


        Ok(())
    }

    pub fn has_clan_higher_permission(&self, db: web::Data<Arc<DbAppState>>, clan_id: Uuid, target_member_id: Uuid) -> Result<(), ApiError> {
        let member = clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(clan_members::user_id.eq(target_member_id))
            .select(ClanMember::as_select())
            .first::<ClanMember>(&mut db.connection()?)
            .optional()?;

        if member.is_some() {
            self.has_clan_permission(db, clan_id, member.unwrap().role)?;
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