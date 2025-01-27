use std::sync::Arc;
use actix_web::web;
use chrono::NaiveDateTime;
use diesel::pg::Pg;
use diesel::{BoxableExpression, ExpressionMethods, PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::expression::AsExpression;
use diesel::sql_types::Bool;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
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

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpdate {
    pub global_name: Option<String>,
    pub description: Option<String>,
    pub country: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceholderOptions {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserListQueryOptions {
    pub name_filter: Option<String>,
    pub placeholder: Option<bool>
}

#[derive(Serialize, Debug)]
pub struct UserPage {
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
        updates: UserUpdate,
    ) -> Result<Self, ApiError> {
        let updated_user = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(&updates)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(updated_user)
    }
}

