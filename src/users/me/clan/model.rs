use crate::app_data::db::DbConnection;
use crate::clans::{Clan, ClanMember};
use crate::error_handler::ApiError;
use crate::schema::clan_members;
use diesel::{
    delete, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use uuid::Uuid;

impl Clan {
    pub fn leave(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
        let member = clan_members::table
            .filter(clan_members::user_id.eq(user_id))
            .select(ClanMember::as_select())
            .first::<ClanMember>(conn)
            .optional()?;

        if !member.is_some() {
            return Err(ApiError::new(404, "You are not part of a clan"));
        }

        if member.unwrap().role == 2 {
            return Err(ApiError::new(403, "You can not leave a clan you're the owner of. You need to either transfer ownership first, or kick all other members and delete the clan."));
        }

        delete(clan_members::table)
            .filter(clan_members::user_id.eq(user_id))
            .execute(conn)?;
        Ok(())
    }
}
