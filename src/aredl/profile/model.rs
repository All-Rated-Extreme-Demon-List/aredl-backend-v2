use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{aredl_levels, aredl_levels_created, aredl_pack_tiers, aredl_packs, aredl_records, roles, user_roles, users};
use crate::custom_schema::{aredl_completed_packs, aredl_user_leaderboard};

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
    pub discord_id: Option<String>,
    pub placeholder: bool,
    pub country: Option<i32>,
    pub description: Option<String>,
    pub ban_level: i32,
    pub discord_avatar: Option<String>,
    pub discord_banner: Option<String>,
    pub discord_accent_color: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=roles)]
pub struct Role {
    pub id: i32,
    pub privilege_level: i32,
    pub role_desc: String
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_user_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub raw_rank: i32,
    pub extremes_rank: i32,
    pub country_rank: i32,
    pub country_raw_rank: i32,
    pub country_extremes_rank: i32,
    pub total_points: i32,
    pub pack_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_levels)]
pub struct Level {
    pub id: Uuid,
    pub level_id: i32,
    pub two_player: bool,
    pub position: i32,
    pub name: String,
    pub points: i32,
    pub legacy: bool,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_records)]
pub struct Record {
    pub id: Uuid,
    pub mobile: bool,
    #[serde(skip_serializing)]
    pub placement_order: i32,
    pub ldm_id: Option<i32>,
    pub video_url: String,
    pub raw_url: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ResolvedRecord {
    #[serde(flatten)]
    pub record: Record,
    pub level: Level,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_packs)]
pub struct Pack {
    pub id: Uuid,
    pub name: String,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug)]
#[diesel(table_name=aredl_pack_tiers)]
pub struct PackTier {
    pub id: Uuid,
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PackResolved {
    #[serde(flatten)]
    pub pack: Pack,
    pub tier: PackTier,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProfileResolved {
    #[serde(flatten)]
    pub user: User,
    pub roles: Vec<Role>,
    pub rank: Option<Rank>,
    pub packs: Vec<PackResolved>,
    pub records: Vec<ResolvedRecord>,
    pub verified: Vec<ResolvedRecord>,
    pub created: Vec<Level>,
    pub published: Vec<Level>,
}

impl ProfileResolved {
    pub fn find(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let user = users::table
            .filter(users::id.eq(id))
            .select(User::as_select())
            .get_result::<User>(conn)?;

        let roles = roles::table
            .inner_join(user_roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(id))
            .order(roles::privilege_level.desc())
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let rank = aredl_user_leaderboard::table
            .filter(aredl_user_leaderboard::user_id.eq(id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let full_records = aredl_records::table
            .filter(aredl_records::submitted_by.eq(id))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_records::level_id)))
            .order(aredl_levels::position.asc())
            .select((Record::as_select(), Level::as_select()))
            .load::<(Record, Level)>(conn)?
            .into_iter()
            .map(|(record, level)| ResolvedRecord {
                record, level
            })
            .collect::<Vec<_>>();

        let (records, verified) = full_records.into_iter()
            .partition(|record| record.record.placement_order != 0);

        let created = aredl_levels::table
            .inner_join(aredl_levels_created::table.on(aredl_levels_created::level_id.eq(aredl_levels::id)))
            .order(aredl_levels::position.asc())
            .filter(aredl_levels_created::user_id.eq(id))
            .select(Level::as_select())
            .load::<Level>(conn)?;

        let published = aredl_levels::table
            .filter(aredl_levels::publisher_id.eq(id))
            .order(aredl_levels::position.asc())
            .select(Level::as_select())
            .load::<Level>(conn)?;

        let packs = aredl_packs::table
            .inner_join(aredl_completed_packs::table.on(aredl_completed_packs::pack_id.eq(aredl_packs::id)))
            .inner_join(aredl_pack_tiers::table.on(aredl_pack_tiers::id.eq(aredl_packs::tier)))
            .filter(aredl_completed_packs::user_id.eq(id))
            .order(aredl_pack_tiers::placement.asc())
            .select((Pack::as_select(), PackTier::as_select()))
            .load::<(Pack, PackTier)>(conn)?
            .into_iter()
            .map(|(pack, tier)| PackResolved { pack, tier })
            .collect::<Vec<_>>();

        Ok(Self { user, roles, rank, packs, records, verified, created, published })
    }
}