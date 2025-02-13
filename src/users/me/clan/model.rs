use diesel::{delete, ExpressionMethods, RunQueryDsl};
use uuid::Uuid;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::clans::Clan;
use crate::schema::clan_members;

impl Clan {

	pub fn leave(conn: &mut DbConnection, user_id: Uuid) -> Result<(), ApiError> {
		delete(clan_members::table)
			.filter(clan_members::user_id.eq(user_id))
			.execute(conn)?;
		Ok(())
	}

}