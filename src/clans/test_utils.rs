#[cfg(test)]
use std::iter::repeat_with;
#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::app_data::db::DbAppState;
#[cfg(test)]
use crate::{
    clans::Clan,
    clans::ClanMember,
    schema::{clan_invites, clan_members, clans},
};
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(test)]
use diesel::{ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
fn generate_random_tag() -> String {
    repeat_with(|| rand::random_range(b'A'..=b'Z') as char)
        .take(5)
        .collect()
}

#[cfg(test)]
pub async fn create_test_clan(db: &Arc<DbAppState>) -> Uuid {
    let conn = &mut db.connection().unwrap();
    loop {
        let tag = generate_random_tag();
        match diesel::insert_into(clans::table)
            .values((clans::global_name.eq("Test Clan"), clans::tag.eq(&tag)))
            .returning(clans::id)
            .get_result(conn)
        {
            Ok(id) => return id,
            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {}
            Err(e) => panic!("Failed to create clan: {e:?}"),
        }
    }
}

#[cfg(test)]
pub async fn create_named_test_clan(
    db: &Arc<DbAppState>,
    global_name: &str,
    tag: &str,
    description: Option<&str>,
) -> Clan {
    diesel::insert_into(clans::table)
        .values((
            clans::global_name.eq(global_name),
            clans::tag.eq(tag),
            clans::description.eq(description),
        ))
        .returning(Clan::as_returning())
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to create named test clan")
}

#[cfg(test)]
pub async fn create_test_clan_member(
    db: &Arc<DbAppState>,
    clan_id: Uuid,
    user_id: Uuid,
    role: i32,
) {
    let conn = &mut db.connection().unwrap();
    diesel::insert_into(clan_members::table)
        .values((
            clan_members::clan_id.eq(clan_id),
            clan_members::user_id.eq(user_id),
            clan_members::role.eq(role),
        ))
        .execute(conn)
        .expect("Failed to add clan member");
}

#[cfg(test)]
pub fn count_test_clan_members(db: &Arc<DbAppState>, clan_id: Uuid, user_id: Uuid) -> i64 {
    clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to count test clan members")
}

#[cfg(test)]
pub fn count_test_clan_invites_for_user(db: &Arc<DbAppState>, user_id: Uuid) -> i64 {
    clan_invites::table
        .filter(clan_invites::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to count test clan invites")
}

#[cfg(test)]
pub fn test_clan_member_user_ids(db: &Arc<DbAppState>, clan_id: Uuid) -> Vec<Uuid> {
    clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .select(clan_members::user_id)
        .load(&mut db.connection().unwrap())
        .expect("Failed to list test clan member user IDs")
}

#[cfg(test)]
pub fn set_test_clan_member_timestamps(
    db: &Arc<DbAppState>,
    clan_id: Uuid,
    user_id: Uuid,
    timestamp: chrono::DateTime<chrono::Utc>,
) {
    diesel::update(
        clan_members::table
            .filter(clan_members::clan_id.eq(clan_id))
            .filter(clan_members::user_id.eq(user_id)),
    )
    .set((
        clan_members::created_at.eq(timestamp),
        clan_members::updated_at.eq(timestamp),
    ))
    .execute(&mut db.connection().unwrap())
    .expect("Failed to set test clan member timestamps");
}

#[cfg(test)]
pub fn test_clan_member(db: &Arc<DbAppState>, clan_id: Uuid, user_id: Uuid) -> ClanMember {
    clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .select(ClanMember::as_select())
        .first(&mut db.connection().unwrap())
        .expect("Failed to get test clan member")
}

#[cfg(test)]
pub async fn create_test_clan_invite(
    db: &Arc<DbAppState>,
    clan_id: Uuid,
    user_id: Uuid,
    invited_by: Uuid,
) -> Uuid {
    let conn = &mut db.connection().unwrap();
    diesel::insert_into(clan_invites::table)
        .values((
            clan_invites::clan_id.eq(clan_id),
            clan_invites::user_id.eq(user_id),
            clan_invites::invited_by.eq(invited_by),
        ))
        .returning(clan_invites::id)
        .get_result(conn)
        .expect("Failed to create clan invite")
}
