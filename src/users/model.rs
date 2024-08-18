use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::{RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::users;

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
    pub discord_id: Option<String>,
    pub placeholder: bool,
    pub description: Option<String>,
    pub country: Option<i32>,
    pub ban_level: i32,
    pub discord_avatar: Option<String>,
    pub discord_banner: Option<String>,
    pub discord_accent_color: Option<i32>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpsert {
    pub username: String,
    pub global_name: String,
    pub discord_id: Option<String>,
    pub placeholder: bool,
    pub country: Option<i32>,
    pub discord_avatar: Option<String>,
    pub discord_banner: Option<String>,
    pub discord_accent_color: Option<i32>,
}

impl User {
    pub fn upsert(db: web::Data<Arc<DbAppState>>, user_upsert: UserUpsert) -> Result<Self, ApiError> {
        let user = diesel::insert_into(users::table)
            .values(&user_upsert)
            .on_conflict(users::username)
            .do_update()
            .set(&user_upsert)
            .returning(Self::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(user)
    }
}

