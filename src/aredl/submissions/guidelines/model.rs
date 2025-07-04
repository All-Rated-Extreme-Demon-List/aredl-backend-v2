use crate::{
    db::DbConnection,
    error_handler::ApiError,
    schema::{aredl::guideline_updates, users},
    users::BaseUser
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, ExpressionMethods, RunQueryDsl, Selectable, QueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable, Insertable, Selectable, Debug, ToSchema, Clone)]
#[diesel(table_name = guideline_updates, check_for_backend(Pg))]
pub struct GuidelineUpdate {
    pub id: Uuid,
    pub text: String,
    pub moderator: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema)]
pub struct GuidelineUpdateFull {
    pub id: Uuid,
    pub text: String,
    pub moderator: BaseUser,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct GuidelineUpdateBody {
    pub guidelines: String
}

impl GuidelineUpdate {
    fn upgrade(update: Self, conn: &mut DbConnection) -> Result<GuidelineUpdateFull, ApiError> {
        let moderator = users::table
            .filter(users::id.eq(update.moderator))
            .select(BaseUser::as_select())
            .first::<BaseUser>(conn)?;

        Ok(GuidelineUpdateFull {
            id: update.id,
            text: update.text,
            created_at: update.created_at,
            moderator,
        })
    }
    pub fn update(
        conn: &mut DbConnection,
        text: String,
        user_id: Uuid
    ) -> Result<GuidelineUpdateFull, ApiError> {
        let update = diesel::insert_into(guideline_updates::table)
            .values((
                guideline_updates::text.eq(text),
                guideline_updates::moderator.eq(user_id)
            ))
            .returning(GuidelineUpdate::as_select())
            .get_result::<GuidelineUpdate>(conn)?;

        Ok(Self::upgrade(update, conn)?)
    }

    pub fn latest(
        conn: &mut DbConnection
    ) -> Result<GuidelineUpdateFull, ApiError> {
        let update = guideline_updates::table
            .order(guideline_updates::created_at.desc())
            .select(Self::as_select())
            .first::<Self>(conn)?;

        Ok(Self::upgrade(update, conn)?)
    }
}