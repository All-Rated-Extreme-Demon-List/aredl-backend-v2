use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::{BoxableExpression, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::expression::AsExpression;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use utoipa::ToSchema;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{users, roles};

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct BaseUser {
    /// Internal UUID of the user.
    pub id: Uuid,
    /// Username of the user. For non-placeholder users, this is linked to the Discord username.
    pub username: String,
    /// Global display name of the user. May be freely set by the user.
    pub global_name: String,
}

#[derive(Serialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct BaseDiscordUser {
    /// Internal UUID of the user.
    pub id: Uuid,
    /// Username of the user. This is linked to the Discord username and is updated on every login.
    pub username: String,
    /// Global display name of the user. May be freely set by the user.
    pub global_name: String,
    /// Discord ID of the user. Updated on every login.
    pub discord_id: Option<String>,
    /// Discord avatar hash of the user. Updated on every login.
    pub discord_avatar: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct User {
    /// Internal UUID of the user.
    pub id: Uuid,
    /// Username of the user. For non-placeholder users, this is linked to the Discord username.
    pub username: String,
    /// Global display name of the user. May be freely set by the user.
    pub global_name: String,
    /// Discord ID of the user. Updated on every login.
    pub discord_id: Option<String>,
    /// Whether the user is a placeholder user or not.
    pub placeholder: bool,
    /// Description of the user. May be freely set by the user.
    pub description: Option<String>,
    /// Country of the user. Uses the ISO 3166-1 numeric country code.
    pub country: Option<i32>,
    /// Ban level of the user.
    pub ban_level: i32,
    /// Discord avatar hash of the user. Updated on every login.
    pub discord_avatar: Option<String>,
    /// Discord banner hash of the user. Updated on every login.
    pub discord_banner: Option<String>,
    /// Discord accent color of the user. Updated on every login.
    pub discord_accent_color: Option<i32>,
    /// Timestamp of when the user was created.
    pub created_at: NaiveDateTime,
    /// Last time the user's tokens were invalidated.
    pub access_valid_after: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
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

#[derive(Serialize, Debug, ToSchema)]
pub struct UserResolved {
    #[serde(flatten)]
    pub user: User,
    /// Roles the user has.
    pub roles: Vec<Role>,
    /// Permissions scopes the user has.
    pub scopes: Vec<String>,
}

#[derive(Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug, ToSchema)]
#[diesel(table_name = roles)]
pub struct Role {
    /// Internal UUID of the role.
    pub id: i32,
    /// Privilege level of the role. Refer to [API Overview](#overview) for more information.
    pub privilege_level: i32,
    /// Name of the role.
    pub role_desc: String,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpdate {
    /// New global display name of the user.
    pub global_name: Option<String>,
    /// New description of the user.
    pub description: Option<String>,
    /// New country of the user. Uses the ISO 3166-1 numeric country code.
    pub country: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserBanUpdate {
    /// New ban level of the user.
    pub ban_level: i32,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlaceholderOptions {
    /// Username of the placeholder to create. Will also be used as the global name.
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserListQueryOptions {
    pub name_filter: Option<String>,
    pub placeholder: Option<bool>
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserPage {
    /// List of found users
    pub data: Vec<User>
}

impl User {
    pub fn upsert(db: web::Data<Arc<DbAppState>>, user_upsert: UserUpsert) -> Result<Self, ApiError> {
        let user = diesel::insert_into(users::table)
            .values(&user_upsert)
            .on_conflict(users::discord_id)
            .do_update()
            .set(&user_upsert)
            .returning(Self::as_select())
            .get_result::<Self>(&mut db.connection()?)?;
        Ok(user)
    }

    pub fn find<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: UserListQueryOptions)
        -> Result<Paginated<UserPage>, ApiError>
    {
        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.name_filter.clone() {
                Some(filter) => Box::new(users::global_name.ilike(filter)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
            };
        let placeholder_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.placeholder.clone() {
                Some(placeholder) => Box::new(users::placeholder.eq(placeholder)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true))
            };

        let entries =
            users::table
                .filter(name_filter)
                .filter(placeholder_filter)
                .order(users::username)
                .limit(page_query.per_page())
                .offset(page_query.offset())
                .select(User::as_select())
                .load::<User>(conn)?;

        let name_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.name_filter {
                Some(filter) => Box::new(users::global_name.ilike(filter)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true)),
            };
        let placeholder_filter: Box<dyn BoxableExpression<_, _, SqlType = Bool>> =
            match options.placeholder {
                Some(placeholder) => Box::new(users::placeholder.eq(placeholder)),
                None => Box::new(<bool as AsExpression<Bool>>::as_expression(true))
            };

        let count = users::table
            .filter(name_filter)
            .filter(placeholder_filter)
            .count()
            .get_result(conn)?;

        Ok(Paginated::<UserPage>::from_data(page_query, count, UserPage {
            data: entries
        }))
    }

    pub fn create_placeholder(
        conn: &mut DbConnection,
        options: PlaceholderOptions,
    ) -> Result<Self, ApiError> {
        let user_data = UserUpsert {
            username: options.username.clone(),
            global_name: options.username,
            placeholder: true,
            discord_id: None,
            country: None,
            discord_avatar: None,
            discord_banner: None,
            discord_accent_color: None,
        };

        let user = diesel::insert_into(users::table)
            .values(&user_data)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;

        Ok(user)
    }

    pub fn update(
        conn: &mut DbConnection,
        user_id: Uuid,
        user: UserUpdate,
    ) -> Result<Self, ApiError> {
        let updated_user = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(&user)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(updated_user)
    }

    pub fn ban(
        conn: &mut DbConnection,
        user_id: Uuid,
        ban_level: i32,
    ) -> Result<User, ApiError> {
        let user = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::ban_level.eq(ban_level))
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(user)
    }
}

