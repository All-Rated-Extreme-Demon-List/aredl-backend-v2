use crate::app_data::db::DbConnection;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::aredl::records::ResolvedRecord;
use crate::clans::Clan;
use crate::error_handler::ApiError;
use crate::schema::{
    aredl::{clans_leaderboard, levels, min_placement_clans_records},
    clan_members, clans, users,
};
use crate::users::{BaseUser, ExtendedBaseUser};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=clans_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=min_placement_clans_records, check_for_backend(Pg))]
pub struct ClanProfileRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// Internal UUID of the submission this record is linked to.
    pub submission_id: Uuid,
    /// Internal UUID of the level the record is for.
    pub level_id: Uuid,
    /// Internal UUID of the user who submitted the record.
    pub submitted_by: Uuid,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record is a verification or not.
    pub is_verification: bool,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedClanProfileLevel {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    pub publisher: BaseUser,
}
#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ClanProfileResolved {
    /// This profile's clan.
    pub clan: Clan,
    /// Rank of the clan in the clans leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from this clan.
    pub records: Vec<ResolvedRecord>,
    /// Levels published by users from this clan.
    pub published: Vec<ResolvedClanProfileLevel>,
}

impl ResolvedRecord {
    pub fn from_clan_data(
        record: ClanProfileRecord,
        level: ExtendedBaseLevel,
        user: ExtendedBaseUser,
    ) -> Self {
        Self {
            id: record.id,
            submission_id: record.submission_id,
            submitted_by: user,
            level,
            mobile: record.mobile,
            video_url: record.video_url,
            is_verification: record.is_verification,
            hide_video: record.hide_video,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

impl ClanProfileResolved {
    pub fn find(conn: &mut DbConnection, clan_id: Uuid) -> Result<Self, ApiError> {
        let clan = clans::table
            .filter(clans::id.eq(clan_id))
            .select(Clan::as_select())
            .first(conn)?;

        let rank = clans_leaderboard::table
            .filter(clans_leaderboard::clan_id.eq(clan_id))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let records = min_placement_clans_records::table
            .filter(min_placement_clans_records::clan_id.eq(clan_id))
            .inner_join(users::table.on(users::id.eq(min_placement_clans_records::submitted_by)))
            .inner_join(levels::table.on(levels::id.eq(min_placement_clans_records::level_id)))
            .select((
                ClanProfileRecord::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(ClanProfileRecord, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, user, level)| ResolvedRecord::from_clan_data(record, level, user))
            .collect();

        let published: Vec<ResolvedClanProfileLevel> = levels::table
            .inner_join(users::table.on(users::id.eq(levels::publisher_id)))
            .inner_join(clan_members::table.on(clan_members::user_id.eq(users::id)))
            .filter(clan_members::clan_id.eq(clan_id))
            .order_by(levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(level, user)| ResolvedClanProfileLevel {
                level,
                publisher: user,
            })
            .collect();

        Ok(Self {
            clan,
            rank,
            records,
            published,
        })
    }
}
