use chrono::NaiveDateTime;
use uuid::Uuid;
use diesel::{ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use diesel::pg::Pg;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::aredl::levels::ExtendedBaseLevel;
use crate::schema::{aredl_levels, users};
use crate::custom_schema::{aredl_country_leaderboard, aredl_min_placement_country_records};

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_country_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CountryProfileLevelResolved {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    pub user: BaseUser,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=aredl_min_placement_country_records, check_for_backend(Pg))]
pub struct CountryProfileRecord {
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
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
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
pub struct CountryProfileRecordResolved {
    #[serde(flatten)]
    pub record: CountryProfileRecord,
    pub user: BaseUser,
    pub level: ExtendedBaseLevel,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CountryProfileResolved {
    /// Country of the profile. Uses the ISO 3166-1 numeric country code.
    pub country: i32,
    /// Rank of the country in the countries leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from the country.
    pub records: Vec<CountryProfileRecordResolved>,
    /// Verification of users from the country.
    pub verified: Vec<CountryProfileRecordResolved>,
    /// Levels published by users from the country.
    pub published: Vec<CountryProfileLevelResolved>,
}

impl CountryProfileResolved {
    pub fn find(conn: &mut DbConnection, country: i32) -> Result<Self, ApiError> {
        let rank = aredl_country_leaderboard::table
            .filter(aredl_country_leaderboard::country.eq(country))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let (verified, records ): (Vec<_>, Vec<_>) = aredl_min_placement_country_records::table
            .filter(aredl_min_placement_country_records::country.eq(country))
            .inner_join(users::table.on(users::id.eq(aredl_min_placement_country_records::submitted_by)))
            .inner_join(aredl_levels::table.on(aredl_levels::id.eq(aredl_min_placement_country_records::level_id)))
            .select((
                CountryProfileRecord::as_select(),
                BaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(CountryProfileRecord, BaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, user, level)| CountryProfileRecordResolved { record, user, level })
            .partition(|resolved| resolved.record.is_verification);
        
        let published: Vec<CountryProfileLevelResolved> = aredl_levels::table
            .inner_join(users::table.on(users::id.eq(aredl_levels::publisher_id)))
            .filter(users::country.eq(country))
            .order_by(aredl_levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(level, user)| CountryProfileLevelResolved { level, user })
            .collect();
        
        Ok(Self { country, rank, records, verified, published })
    }
}
