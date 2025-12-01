use crate::app_data::db::DbConnection;
use crate::clans::Clan;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{clan_members, clans, permissions, roles, user_roles, users};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::pg::Pg;
use diesel::{
    BoolExpressionMethods, ExpressionMethods, JoinOnDsl, OptionalExtension,
    PgTextExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
pub struct BaseUserWithBanLevel {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
    pub ban_level: i32,
}

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

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct ExtendedBaseUser {
    /// Internal UUID of the user.
    pub id: Uuid,
    /// Username of the user. For non-placeholder users, this is linked to the Discord username.
    pub username: String,
    /// Global display name of the user. May be freely set by the user.
    pub global_name: String,
    /// Country of the user. Uses the ISO 3166-1 numeric country code.
    pub country: Option<i32>,
    /// Discord ID of the user. Updated on every login.
    pub discord_id: Option<String>,
    /// Discord avatar hash of the user. Updated on every login.
    pub discord_avatar: Option<String>,
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
    pub created_at: DateTime<Utc>,
    // Last time the user's tokens were invalidated.
    #[serde(skip_serializing)]
    pub access_valid_after: DateTime<Utc>,
    /// The level the user has beaten and chosen as their profile background.
    pub background_level: i32,
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
    pub last_discord_avatar_update: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpdateOnLogin {
    pub username: String,
    pub discord_id: Option<String>,
    pub discord_avatar: Option<String>,
    pub discord_banner: Option<String>,
    pub discord_accent_color: Option<i32>,
    pub last_discord_avatar_update: Option<NaiveDateTime>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserResolved {
    #[serde(flatten)]
    pub user: User,
    /// Clan the user is in.
    pub clan: Option<Clan>,
    /// Roles the user has.
    pub roles: Vec<Role>,
    /// Permissions scopes the user has.
    pub scopes: Vec<String>,
}

#[derive(
    Serialize, Deserialize, Queryable, Selectable, Identifiable, PartialEq, Debug, ToSchema,
)]
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
    pub placeholder: Option<bool>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserPage {
    /// List of found users
    pub data: Vec<User>,
}

// user filter that matches either by UUID, username, or discord_id
pub fn user_filter<'a>(input: &'a String) -> users::BoxedQuery<'a, Pg> {
    let mut q = users::table.into_boxed::<Pg>();

    if let Ok(uuid) = Uuid::parse_str(&input) {
        q = q.filter(users::id.eq(uuid));
    } else {
        q = q.filter(
            users::discord_id
                .eq(Some(input.to_owned()))
                .or(users::username.eq(input)),
        );
    }

    q
}

impl BaseUser {
    pub fn from_base_user_with_ban_level(user: BaseUserWithBanLevel) -> Self {
        BaseUser {
            id: user.id,
            username: if user.ban_level == 3 {
                "REDACTED".to_string()
            } else {
                user.username
            },
            global_name: if user.ban_level == 3 {
                "REDACTED".to_string()
            } else {
                user.global_name
            },
        }
    }
}

impl User {
    pub fn from_uuid(conn: &mut DbConnection, user_id: Uuid) -> Result<Self, ApiError> {
        Ok(users::table
            .filter(users::id.eq(user_id))
            .select(User::as_select())
            .first::<User>(conn)?)
    }

    pub fn from_str(conn: &mut DbConnection, user_id: &str) -> Result<Self, ApiError> {
        match Uuid::parse_str(user_id) {
            Ok(uuid) => Ok(Self::from_uuid(conn, uuid)?),
            Err(_) => Ok(users::table
                .filter(
                    users::discord_id
                        .eq(Some(user_id.to_owned()))
                        .or(users::username.eq(user_id.to_owned())),
                )
                .select(User::as_select())
                .first::<User>(conn)?),
        }
    }

    pub fn is_banned(user_id: Uuid, conn: &mut DbConnection) -> Result<bool, ApiError> {
        let user = users::table
            .filter(users::id.eq(user_id))
            .select(users::ban_level)
            .first::<i32>(conn)
            .optional()?;

        match user {
            Some(ban_level) => Ok(ban_level > 1),
            None => Err(ApiError::new(404, "User not found")),
        }
    }

    pub fn upsert(conn: &mut DbConnection, user_upsert: UserUpsert) -> Result<Self, ApiError> {
        let existing_user = users::table
            .filter(users::discord_id.eq(user_upsert.discord_id.clone()))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()?;

        match existing_user {
            Some(user) => {
                let updated_user = diesel::update(users::table.filter(users::id.eq(user.id)))
                    .set(UserUpdateOnLogin {
                        username: user_upsert.username.clone(),
                        discord_id: user_upsert.discord_id.clone(),
                        discord_avatar: user_upsert.discord_avatar,
                        discord_banner: user_upsert.discord_banner,
                        discord_accent_color: user_upsert.discord_accent_color,
                        last_discord_avatar_update: Some(Utc::now().naive_utc()),
                    })
                    .returning(Self::as_select())
                    .get_result::<Self>(conn)?;
                return Ok(updated_user);
            }
            None => {
                let user = diesel::insert_into(users::table)
                    .values(&user_upsert)
                    .returning(Self::as_select())
                    .get_result::<Self>(conn)?;
                return Ok(user);
            }
        }
    }

    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: UserListQueryOptions,
    ) -> Result<Paginated<UserPage>, ApiError> {
        let build_query = || {
            let mut q = users::table.into_boxed::<Pg>();
            if let Some(ref name_like) = options.name_filter {
                q = q.filter(
                    users::global_name.ilike(name_like).or(users::username
                        .ilike(name_like)
                        .or(users::id.eq_any(user_filter(name_like).select(users::id)))),
                );
            }
            if let Some(placeholder) = options.placeholder {
                q = q.filter(users::placeholder.eq(placeholder));
            }
            q
        };

        let total_count: i64 = build_query().count().get_result(conn)?;
        let mut q = build_query();

        if let Some(ref name_like) = options.name_filter {
            q = q.order((
                users::username.eq(name_like).desc(),
                users::global_name.eq(name_like).desc(),
                users::username.asc(),
            ));
        } else {
            q = q.order(users::username.asc());
        }

        let entries: Vec<User> = q
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(User::as_select())
            .load(conn)?;

        Ok(Paginated::from_data(
            page_query,
            total_count,
            UserPage { data: entries },
        ))
    }

    pub fn create_placeholder(
        conn: &mut DbConnection,
        options: PlaceholderOptions,
    ) -> Result<Self, ApiError> {
        let user = diesel::insert_into(users::table)
            .values((
                users::placeholder.eq(true),
                users::global_name.eq(options.username),
            ))
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

    pub fn ban(conn: &mut DbConnection, user_id: Uuid, ban_level: i32) -> Result<User, ApiError> {
        let user = diesel::update(users::table.filter(users::id.eq(user_id)))
            .set(users::ban_level.eq(ban_level))
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(user)
    }
}

impl From<User> for BaseUser {
    fn from(user: User) -> Self {
        BaseUser {
            id: user.id,
            username: user.username,
            global_name: user.global_name,
        }
    }
}

impl UserResolved {
    pub fn from_uuid(conn: &mut DbConnection, uuid: Uuid) -> Result<Self, ApiError> {
        let user = User::from_uuid(conn, uuid)?;
        Self::from_user(conn, user)
    }

    pub fn from_str(conn: &mut DbConnection, user_id: &str) -> Result<Self, ApiError> {
        let user = User::from_str(conn, user_id)?;
        Self::from_user(conn, user)
    }

    pub fn from_user(conn: &mut DbConnection, user: User) -> Result<Self, ApiError> {
        let clan = clans::table
            .inner_join(clan_members::table.on(clans::id.eq(clan_members::clan_id)))
            .filter(clan_members::user_id.eq(user.id))
            .select(Clan::as_select())
            .first::<Clan>(conn)
            .optional()?;

        let roles = user_roles::table
            .inner_join(roles::table.on(user_roles::role_id.eq(roles::id)))
            .filter(user_roles::user_id.eq(user.id))
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let user_privilege_level: i32 = roles
            .iter()
            .map(|role| role.privilege_level)
            .max()
            .unwrap_or(0);

        let all_permissions = permissions::table
            .select((permissions::permission, permissions::privilege_level))
            .load::<(String, i32)>(conn)?;

        let scopes = all_permissions
            .into_iter()
            .filter_map(|(permission, privilege_level)| {
                if user_privilege_level >= privilege_level {
                    Some(permission)
                } else {
                    None
                }
            })
            .collect::<Vec<String>>();
        Ok(UserResolved {
            user,
            clan,
            roles,
            scopes,
        })
    }
}
