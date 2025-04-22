use chrono::NaiveDateTime;
use uuid::Uuid;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use diesel::pg::Pg;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::clans::Clan;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::schema::{aredl_levels, clan_members, clans, users};
use crate::custom_schema::{aredl_clans_leaderboard, aredl_min_placement_clans_records};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_clans_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanProfileLevelResolved {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    pub user: BaseUser,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_min_placement_clans_records, check_for_backend(Pg))]
pub struct ClanProfileRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Internal UUID of the level the record is for.
    pub level_id: Uuid,
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    #[serde(skip_serializing)]
    pub is_verification: bool,
    /// Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: NaiveDateTime,
    /// Timestamp of when the record was last updated.
    pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanProfileRecordResolved {
    #[serde(flatten)]
    pub record: ClanProfileRecord,
    pub user: BaseUser,
    pub level: ExtendedBaseLevel,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanProfileResolved {
    /// This profile's clan.
    pub clan: Clan,
    /// Rank of the clan in the clans leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from this clan.
    pub records: Vec<ClanProfileRecordResolved>,
    /// Verification of users from this clan.
    pub verified: Vec<ClanProfileRecordResolved>,
    /// Levels published by users from this clan.
    pub published: Vec<ClanProfileLevelResolved>,
}

impl ClanProfileResolved {
    pub fn find(conn: &mut DbConnection, clan_id: Uuid) -> Result<Self, ApiError> {

        let clan = clans::table
            .filter(clans::id.eq(clan_id))
            .select(Clan::as_select())
            .first(conn)?;

        let rank = aredl_clans_leaderboard::table
            .filter(aredl_clans_leaderboard::clan_id.eq(clan_id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let (records, verified): (Vec<_>, Vec<_>) = aredl_min_placement_clans_records::table
            .filter(aredl_min_placement_clans_records::clan_id.eq(clan_id))
            .inner_join(users::table.on(users::id.eq(aredl_min_placement_clans_records::submitted_by)))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_min_placement_clans_records::level_id)))
            .select((
                ClanProfileRecord::as_select(),
                BaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(ClanProfileRecord, BaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, user, level)| ClanProfileRecordResolved { record, user, level })
            .partition(|resolved| resolved.record.is_verification);
        
        let published: Vec<ClanProfileLevelResolved> = aredl_levels::table
            .inner_join(users::table.on(users::id.eq(aredl_levels::publisher_id)))
            .inner_join(clan_members::table.on(clan_members::user_id.eq(users::id)))
            .filter(clan_members::clan_id.eq(clan_id))
            .order_by(aredl_levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(level, user)| ClanProfileLevelResolved { level, user })
            .collect();
        
        Ok(Self { clan, rank, records, verified, published })
    }
}
