use crate::app_data::db::DbConnection;
use crate::audit::AuditLogEntry;
use crate::clans::{Clan, ClanMember};
use crate::error_handler::ApiError;
use crate::schema::clan_members;
use diesel::delete;
use uuid::Uuid;

use diesel::prelude::*;
impl Clan {
    pub fn leave(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        conn.transaction(|conn| {
            let member = clan_members::table
                .filter(clan_members::user_id.eq(user_id))
                .select(ClanMember::as_select())
                .first::<ClanMember>(conn)
                .optional()?
                .ok_or(ApiError::NotFound("You are not part of a clan"))?;

            if member.role == 2 {
                return Err(ApiError::Conflict(
                    "You can not leave a clan you're the owner of. You need to either transfer ownership first, or kick all other members and delete the clan.",
                ));
            }

            let member = delete(clan_members::table)
                .filter(clan_members::user_id.eq(user_id))
                .returning(ClanMember::as_select())
                .get_result::<ClanMember>(conn)?;
            AuditLogEntry::log_delete(conn, Some(user_id), &member)?;
            Ok(())
        })
    }
}
