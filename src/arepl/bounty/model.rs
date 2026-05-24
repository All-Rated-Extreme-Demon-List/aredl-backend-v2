use crate::app_data::db::DbConnection;
use crate::arepl::levels::ExtendedBaseLevel;
use crate::arepl::records::Record;
use crate::auth::{Authenticated, Permission};
use crate::error_handler::ApiError;
use crate::schema::arepl::{bounties, bounty_completed, levels};
use chrono::{DateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, NullableExpressionMethods,
    QueryDsl, RunQueryDsl, Selectable, SelectableHelper,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[ExistingTypePath = "crate::schema::arepl::sql_types::BountyType"]
#[DbValueStyle = "PascalCase"]
pub enum BountyType {
    Bounty,
    Weekly,
    Monthly,
    Event,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, Copy, PartialEq, Eq, Hash)]
#[ExistingTypePath = "crate::schema::arepl::sql_types::BountyDifficulty"]
#[DbValueStyle = "PascalCase"]
pub enum BountyDifficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Debug, ToSchema)]
#[diesel(table_name=bounties, check_for_backend(Pg))]
pub struct Bounty {
    /// Internal UUID for the bounty.
    pub id: Uuid,
    /// The internal UUID of the level associated with this bounty.
    pub level_id: Uuid,
    /// The subtype of this bounty.
    pub bounty_type: BountyType,
    /// The difficulty range of the level for this bounty.
    pub bounty_difficulty: BountyDifficulty,
    /// The date after which this bounty is active.
    pub start_date: DateTime<Utc>,
    /// The date after which this bounty is closed. This can either be set in advance or left unset for bounties that are closed manually or that will close automatically after reaching a completion threshold.
    pub end_date: Option<DateTime<Utc>>,
    /// The target number of submissions for this bounty. This can be used to automatically close the bounty after a certain number of completions.
    pub target_submissions: Option<i32>,
    /// Whether or not the target number of submissions for this bounty should be displayed publicly, or kept private to staff only.
    pub is_target_public: bool,
}

#[derive(Serialize, ToSchema)]
pub struct BountyResolved {
    /// Internal UUID for the bounty.
    pub id: Uuid,
    /// The level associated with this bounty.
    pub level: ExtendedBaseLevel,
    /// The subtype of this bounty.
    pub bounty_type: BountyType,
    /// The difficulty range of the level for this bounty.
    pub bounty_difficulty: BountyDifficulty,
    /// The date after which this bounty is active.
    pub start_date: DateTime<Utc>,
    /// The date after which this bounty is closed. This can either be set in advance or left unset for bounties that are closed manually or that will close automatically after reaching a completion threshold.
    pub end_date: Option<DateTime<Utc>>,
    /// The target number of submissions for this bounty. This can be used to automatically close the bounty after a certain number of completions.
    pub target_submissions: Option<i32>,
    /// Whether or not the target number of submissions for this bounty should be displayed publicly, or kept private to staff only.
    pub is_target_public: bool,
    /// Whether or not the user has completed this bounty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_by_user: Option<bool>,
}

impl BountyResolved {
    pub fn find_all(
        conn: &mut DbConnection,
        authenticated: Option<Authenticated>,
    ) -> Result<Vec<Self>, ApiError> {
        let user_id = authenticated.as_ref().map(|auth| auth.user_id);

        let base_bounties_query =
            bounties::table.inner_join(levels::table.on(bounties::level_id.eq(levels::id)));

        let bounties = match user_id {
            Some(user) => base_bounties_query
                .left_join(
                    bounty_completed::table.on(bounties::id
                        .eq(bounty_completed::bounty_id)
                        .and(bounty_completed::user_id.eq(user))),
                )
                .select((
                    Bounty::as_select(),
                    ExtendedBaseLevel::as_select(),
                    bounty_completed::user_id.nullable(),
                ))
                .load::<(Bounty, ExtendedBaseLevel, Option<Uuid>)>(conn)?
                .into_iter()
                .map(|(bounty, level, user_completed)| {
                    (bounty, level, Some(user_completed.is_some()))
                })
                .collect::<Vec<_>>(),
            None => base_bounties_query
                .select((Bounty::as_select(), ExtendedBaseLevel::as_select()))
                .load::<(Bounty, ExtendedBaseLevel)>(conn)?
                .into_iter()
                .map(|(bounty, level)| (bounty, level, None))
                .collect::<Vec<_>>(),
        };

        let has_bounty_manage = authenticated.as_ref().map_or(false, |auth| {
            auth.has_permission(conn, Permission::BountyManage)
                .unwrap_or(false)
        });

        Ok(bounties
            .into_iter()
            .map(|(bounty, level, user_completed)| {
                let hide_target = !has_bounty_manage && !bounty.is_target_public;
                Self::from_data(bounty, level, user_completed, hide_target)
            })
            .collect())
    }

    pub fn from_data(
        bounty: Bounty,
        level: ExtendedBaseLevel,
        user_completed: Option<bool>,
        hide_target: bool,
    ) -> Self {
        Self {
            id: bounty.id,
            level,
            bounty_type: bounty.bounty_type,
            bounty_difficulty: bounty.bounty_difficulty,
            start_date: bounty.start_date,
            end_date: bounty.end_date,
            target_submissions: if hide_target {
                None
            } else {
                bounty.target_submissions
            },
            is_target_public: bounty.is_target_public,
            completed_by_user: user_completed,
        }
    }
}

#[derive(Deserialize, Insertable, AsChangeset, ToSchema, Debug)]
#[diesel(table_name=bounties, check_for_backend(Pg))]
pub struct BountyPost {
    /// The internal UUID of the level associated with this bounty.
    pub level_id: Uuid,
    /// The subtype of this bounty.
    pub bounty_type: BountyType,
    /// The difficulty range of the level for this bounty.
    pub bounty_difficulty: BountyDifficulty,
    /// The date after which this bounty is active.
    pub start_date: DateTime<Utc>,
    /// The date after which this bounty is closed. This can either be set in advance or left unset for bounties that are closed manually or that will close automatically after reaching a completion threshold.
    pub end_date: Option<DateTime<Utc>>,
    /// The target number of submissions for this bounty. This can be used to automatically close the bounty after a certain number of completions.
    pub target_submissions: Option<i32>,
    /// Whether or not the target number of submissions for this bounty should be displayed publicly, or kept private to staff only.
    pub is_target_public: bool,
}

#[derive(Deserialize, AsChangeset, ToSchema, Debug)]
#[diesel(table_name=bounties, check_for_backend(Pg))]
pub struct BountyPatch {
    /// The internal UUID of the level associated with this bounty.
    pub level_id: Option<Uuid>,
    /// The subtype of this bounty.
    pub bounty_type: Option<BountyType>,
    /// The difficulty range of the level for this bounty.
    pub bounty_difficulty: Option<BountyDifficulty>,
    /// The date after which this bounty is active.
    pub start_date: Option<DateTime<Utc>>,
    /// The date after which this bounty is closed. This can either be set in advance or left unset for bounties that are closed manually or that will close automatically after reaching a completion threshold.
    pub end_date: Option<DateTime<Utc>>,
    /// The target number of submissions for this bounty. This can be used to automatically close the bounty after a certain number of completions.
    pub target_submissions: Option<i32>,
    /// Whether or not the target number of submissions for this bounty should be displayed publicly, or kept private to staff only.
    pub is_target_public: Option<bool>,
}

impl Bounty {
    pub fn create(conn: &mut DbConnection, new_bounty: BountyPost) -> Result<Self, ApiError> {
        if let Some(end_date) = new_bounty.end_date {
            if end_date <= new_bounty.start_date {
                return Err(ApiError::new(400, "End date must be after start date."));
            }
        }

        if let Some(target) = new_bounty.target_submissions {
            if target <= 0 {
                return Err(ApiError::new(
                    400,
                    "Target submissions must be a positive integer.",
                ));
            }
        }

        let bounty = diesel::insert_into(bounties::table)
            .values(new_bounty)
            .get_result(conn)?;
        Ok(bounty)
    }

    pub fn update(conn: &mut DbConnection, id: Uuid, patch: BountyPatch) -> Result<Self, ApiError> {
        let existing_bounty = bounties::table
            .filter(bounties::id.eq(id))
            .first::<Bounty>(conn)?;

        let start_date = match patch.start_date {
            Some(date) => date,
            None => existing_bounty.start_date,
        };
        let end_date = match patch.end_date {
            Some(date) => Some(date),
            None => existing_bounty.end_date,
        };

        if let Some(end_date) = end_date {
            if end_date <= start_date {
                return Err(ApiError::new(400, "End date must be after start date."));
            }
        }

        if let Some(target) = patch.target_submissions {
            if target <= 0 {
                return Err(ApiError::new(
                    400,
                    "Target submissions must be a positive integer.",
                ));
            }
        }

        let bounty = diesel::update(bounties::table)
            .set(patch)
            .filter(bounties::id.eq(id))
            .get_result(conn)?;
        Ok(bounty)
    }

    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let bounty =
            diesel::delete(bounties::table.filter(bounties::id.eq(id))).get_result(conn)?;
        Ok(bounty)
    }

    pub fn find_active_by_level(
        conn: &mut DbConnection,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let current_time = Utc::now();
        let bounties = bounties::table
            .filter(bounties::level_id.eq(level_id))
            .filter(bounties::start_date.le(current_time))
            .filter(
                bounties::end_date
                    .is_null()
                    .or(bounties::end_date.gt(current_time)),
            )
            .load::<Bounty>(conn)?;
        Ok(bounties)
    }
}

impl Record {
    pub fn complete_bounty_if_exists(&self, conn: &mut DbConnection) -> Result<(), ApiError> {
        let bounties = bounties::table
            .filter(bounties::level_id.eq(self.level_id))
            .filter(bounties::start_date.le(self.achieved_at))
            .filter(
                bounties::end_date
                    .is_null()
                    .or(bounties::end_date.gt(self.achieved_at)),
            )
            .select(bounties::id)
            .load::<Uuid>(conn)?;

        for bounty_id in bounties {
            conn.transaction(|conn| -> Result<(), ApiError> {
                let bounty = bounties::table
                    .filter(bounties::id.eq(bounty_id))
                    .for_update()
                    .first::<Bounty>(conn)?;

                let current_completions = bounty.count_completions(conn)?;

                if let Some(target) = bounty.target_submissions {
                    if current_completions >= target as i64 {
                        return Ok(());
                    }
                }

                diesel::insert_into(bounty_completed::table)
                    .values((
                        bounty_completed::bounty_id.eq(bounty.id),
                        bounty_completed::user_id.eq(self.submitted_by),
                        bounty_completed::completed_at.eq(Utc::now()),
                    ))
                    .on_conflict((bounty_completed::bounty_id, bounty_completed::user_id))
                    .do_nothing()
                    .execute(conn)?;

                if let Some(target) = bounty.target_submissions {
                    if bounty.count_completions(conn)? >= target as i64 {
                        diesel::update(bounties::table.filter(bounties::id.eq(bounty.id)))
                            .set(bounties::end_date.eq(Utc::now()))
                            .execute(conn)?;
                    }
                }

                Ok(())
            })?;
        }

        Ok(())
    }
}
