use crate::app_data::db::DbConnection;
use crate::arepl::levels::ExtendedBaseLevel;
use crate::arepl::records::ResolvedRecord;
use crate::error_handler::ApiError;
use crate::schema::{
    arepl::{country_leaderboard, levels, min_placement_country_records},
    users,
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
#[diesel(table_name=country_leaderboard)]
pub struct Rank {
    pub rank: i32,
    pub extremes_rank: i32,
    pub level_points: i32,
    pub extremes: i32,
}
#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=min_placement_country_records, check_for_backend(Pg))]
pub struct CountryProfileRecord {
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
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
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
pub struct ResolvedCountryProfileLevel {
    #[serde(flatten)]
    pub level: ExtendedBaseLevel,
    /// The user who published the level.
    pub publisher: BaseUser,
}

impl ResolvedRecord {
    pub fn from_country_data(
        record: CountryProfileRecord,
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
            completion_time: record.completion_time,
            is_verification: record.is_verification,
            hide_video: record.hide_video,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct CountryProfileResolved {
    /// Country of the profile. Uses the ISO 3166-1 numeric country code.
    pub country: i32,
    /// Rank of the country in the countries leaderboard.
    pub rank: Option<Rank>,
    /// Records of users from the country.
    pub records: Vec<ResolvedRecord>,
    /// Levels published by users from the country.
    pub published: Vec<ResolvedCountryProfileLevel>,
}

impl CountryProfileResolved {
    pub fn find(conn: &mut DbConnection, country: i32) -> Result<Self, ApiError> {
        let rank = country_leaderboard::table
            .filter(country_leaderboard::country.eq(country))
            .select(Rank::as_select())
            .first(conn)
            .optional()?;

        let records = min_placement_country_records::table
            .filter(min_placement_country_records::country.eq(country))
            .inner_join(users::table.on(users::id.eq(min_placement_country_records::submitted_by)))
            .inner_join(levels::table.on(levels::id.eq(min_placement_country_records::level_id)))
            .select((
                CountryProfileRecord::as_select(),
                ExtendedBaseUser::as_select(),
                ExtendedBaseLevel::as_select(),
            ))
            .load::<(CountryProfileRecord, ExtendedBaseUser, ExtendedBaseLevel)>(conn)?
            .into_iter()
            .map(|(record, user, level)| ResolvedRecord::from_country_data(record, level, user))
            .collect();

        let published = levels::table
            .inner_join(users::table.on(users::id.eq(levels::publisher_id)))
            .filter(users::country.eq(country))
            .order_by(levels::position.asc())
            .select((ExtendedBaseLevel::as_select(), BaseUser::as_select()))
            .load(conn)?
            .into_iter()
            .map(|(level, user)| ResolvedCountryProfileLevel {
                level,
                publisher: user,
            })
            .collect();

        Ok(Self {
            country,
            rank,
            records,
            published,
        })
    }
}
