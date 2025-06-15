#[cfg(test)]
use crate::db::DbConnection;
#[cfg(test)]
use crate::schema::{clan_invites, clan_members, clans};
use diesel::result::{DatabaseErrorKind, Error};
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
use rand::rngs::OsRng;
use rand::TryRngCore;
#[cfg(test)]
use uuid::Uuid;

fn generate_random_tag() -> String {
    let mut rng = OsRng;
    let mut buf = [0u8; 5];
    let _ = rng.try_fill_bytes(&mut buf);
    buf.iter().map(|&b| ((b % 26) + b'A') as char).collect()
}

#[cfg(test)]
pub async fn create_test_clan(conn: &mut DbConnection) -> Uuid {
    loop {
        let tag = generate_random_tag();
        match diesel::insert_into(clans::table)
            .values((clans::global_name.eq("Test Clan"), clans::tag.eq(&tag)))
            .returning(clans::id)
            .get_result(conn)
        {
            Ok(id) => return id,
            Err(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _)) => {
                continue;
            }
            Err(e) => panic!("Failed to create clan: {:?}", e),
        }
    }
}

#[cfg(test)]
pub async fn create_test_clan_member(
    conn: &mut DbConnection,
    clan_id: Uuid,
    user_id: Uuid,
    role: i32,
) {
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
pub async fn create_test_clan_invite(
    conn: &mut DbConnection,
    clan_id: Uuid,
    user_id: Uuid,
    invited_by: Uuid,
) -> Uuid {
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
