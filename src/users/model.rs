use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry, Auditable};
use crate::auth::{permission, Authenticated, Permission};
use crate::clans::Clan;
use crate::error_handler::ApiError;
use crate::list::records::Record;
use crate::list::submissions::SubmissionStatus;
use crate::page_helper::{PageQuery, Paginated};
use crate::roles::Role;
use crate::schema::{
    clan_members, clans, merge_logs, oauth_connected_accounts, oauth_requests, records, roles,
    submissions, user_roles, users,
};
use crate::users::badges::{UserBadge, UserBadgeResolved};
use crate::users::id_resolver::UserIdentifier;
use crate::users::{user_ilike_filter, AccountStatus};
use chrono::{DateTime, NaiveDateTime, Utc};
use diesel::pg::Pg;
use diesel::sql_types::VarChar;
use serde::{Deserialize, Serialize};
use serde_with::rust::double_option;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Queryable, Selectable, Debug)]
#[diesel(table_name = users)]
pub struct BaseUserWithAccountStatus {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
    pub account_status: AccountStatus,
}

#[derive(Debug, Serialize, Deserialize, Queryable, Selectable, ToSchema, Clone)]
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
    /// Discord avatar decoration asset of the user. Updated on every login.
    pub discord_avatar_decoration: Option<String>,
    /// The badge the user has unlocked and chosen to feature on their profile.
    pub featured_badge_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable, ToSchema)]
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
    /// Account status of the user.
    pub account_status: AccountStatus,
    /// Discord avatar hash of the user. Updated on every login.
    pub discord_avatar: Option<String>,
    /// Discord avatar decoration asset of the user. Updated on every login.
    pub discord_avatar_decoration: Option<String>,
    /// Timestamp of when the user was created.
    pub created_at: DateTime<Utc>,
    // Last time the user's tokens were invalidated.
    #[serde(skip_serializing)]
    pub access_valid_after: DateTime<Utc>,
    /// The level the user has beaten and chosen as their profile background.
    pub background_level: Option<i32>,
    /// The badge the user has unlocked and chosen to feature on their profile.
    pub featured_badge_code: Option<String>,
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
    pub discord_avatar_decoration: Option<String>,
    pub last_discord_avatar_update: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, AsChangeset, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpdateOnLogin {
    pub username: String,
    pub discord_id: Option<String>,
    pub discord_avatar: Option<String>,
    #[diesel(treat_none_as_null = true)]
    pub discord_avatar_decoration: Option<String>,
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
    /// All unlocked badges for the user.
    pub badges: Vec<UserBadgeResolved>,
}

#[derive(Debug, Serialize, Deserialize, Insertable, AsChangeset, ToSchema)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct UserUpdate {
    /// New global display name of the user.
    pub global_name: Option<String>,
    /// New description of the user.
    #[serde(default, with = "double_option")]
    pub description: Option<Option<String>>,
    /// New country of the user. Uses the ISO 3166-1 numeric country code.
    #[serde(default, with = "double_option")]
    pub country: Option<Option<i32>>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UserAccountStatusUpdate {
    /// New account status of the user.
    #[schema(minimum = 0, maximum = 4)]
    pub account_status: AccountStatus,
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
    pub account_status: Option<AccountStatus>,
}

#[derive(Serialize, Debug, ToSchema)]
pub struct UserPage {
    /// List of found users
    pub data: Vec<User>,
}

impl BaseUser {
    pub fn hidden() -> Self {
        BaseUser {
            id: "00000000-0000-0000-0000-000000000000"
                .parse()
                .expect("Constant should not fail"),
            username: "Hidden user".to_owned(),
            global_name: "Hidden user".to_owned(),
        }
    }

    pub fn from_base_user_with_account_status(user: BaseUserWithAccountStatus) -> Self {
        if user.account_status == AccountStatus::Redacted {
            BaseUser {
                id: user.id,
                username: "-".to_owned(),
                global_name: "-".to_owned(),
            }
        } else {
            BaseUser {
                id: user.id,
                username: user.username,
                global_name: user.global_name,
            }
        }
    }
}

impl ExtendedBaseUser {
    pub fn hidden() -> Self {
        ExtendedBaseUser {
            id: "00000000-0000-0000-0000-000000000000"
                .parse()
                .expect("Constant should not fail"),
            username: "Hidden user".to_owned(),
            global_name: "Hidden user".to_owned(),
            country: None,
            discord_id: None,
            discord_avatar: None,
            discord_avatar_decoration: None,
            featured_badge_code: None,
        }
    }
}

impl User {
    pub fn find(
        conn: &mut DbConnection,
        identifier: impl Into<UserIdentifier>,
    ) -> Result<Self, ApiError> {
        Ok(identifier
            .into()
            .filter()
            .select(Self::as_select())
            .first(conn)?)
    }

    /// tries to fallback merge logs if the given identifier can't be found in the current users table
    pub fn find_with_merge_fallback(
        conn: &mut DbConnection,
        identifier: impl Into<UserIdentifier>,
    ) -> Result<Self, ApiError> {
        let uuid = match identifier.into() {
            UserIdentifier::Uuid(uuid) => uuid,
            identifier => return Self::find(conn, identifier),
        };

        if let Some(user) = users::table
            .filter(users::id.eq(uuid))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()?
        {
            return Ok(user);
        }

        Ok(merge_logs::table
            .inner_join(users::table)
            .filter(merge_logs::secondary_user.eq(uuid))
            .order((merge_logs::merged_at.desc(), merge_logs::id.desc()))
            .select(Self::as_select())
            .first::<Self>(conn)?)
    }

    pub fn is_banned(user_id: Uuid, conn: &mut DbConnection) -> Result<bool, ApiError> {
        let user = users::table
            .filter(users::id.eq(user_id))
            .select(users::account_status)
            .first::<AccountStatus>(conn)
            .optional()?;

        match user {
            Some(account_status) => Ok(account_status >= AccountStatus::ListBanned),
            None => Err(ApiError::NotFound("User not found")),
        }
    }

    pub fn upsert(conn: &mut DbConnection, user_upsert: UserUpsert) -> Result<Self, ApiError> {
        let existing_user = users::table
            .filter(users::discord_id.eq(user_upsert.discord_id.clone()))
            .select(Self::as_select())
            .first::<Self>(conn)
            .optional()?;

        if let Some(user) = existing_user {
            let updated_user = diesel::update(users::table.filter(users::id.eq(user.id)))
                .set(UserUpdateOnLogin {
                    username: user_upsert.username.clone(),
                    discord_id: user_upsert.discord_id.clone(),
                    discord_avatar: user_upsert.discord_avatar,
                    discord_avatar_decoration: user_upsert.discord_avatar_decoration,
                    last_discord_avatar_update: Some(Utc::now().naive_utc()),
                })
                .returning(Self::as_select())
                .get_result::<Self>(conn)?;

            AuditLogEntry::log_update(conn, Some(user.id), &user, &updated_user)?;
            Ok(updated_user)
        } else {
            let user = diesel::insert_into(users::table)
                .values(&user_upsert)
                .returning(Self::as_select())
                .get_result::<Self>(conn)?;

            AuditLogEntry::log_create(conn, Some(user.id), &user)?;
            Ok(user)
        }
    }

    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: &UserListQueryOptions,
        authenticated: Option<&Authenticated>,
    ) -> Result<Paginated<UserPage>, ApiError> {
        let has_permission = authenticated.is_some_and(|auth| {
            auth.has_permission(conn, Permission::UserBan)
                .unwrap_or(false)
        });
        let build_query = || {
            let mut q = users::table.into_boxed::<Pg>();
            if let Some(name_like) = &options.name_filter {
                q = q.filter(users::id.eq_any(user_ilike_filter(name_like).select(users::id)));
            }
            if let Some(placeholder) = options.placeholder {
                q = q.filter(users::placeholder.eq(placeholder));
            }
            if let Some(account_status) = options.account_status {
                q = q.filter(users::account_status.eq(account_status));
            }

            if !has_permission {
                q = q.filter(users::account_status.eq(AccountStatus::Normal));
            }
            q
        };

        let total_count: i64 = build_query().count().get_result(conn)?;
        let mut q = build_query();

        if let Some(name_like) = &options.name_filter {
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
        user: &UserUpdate,
    ) -> Result<Self, ApiError> {
        let updated_user = diesel::update(
            users::table
                .filter(users::id.eq(user_id))
                .filter(users::account_status.ne(AccountStatus::Deleted)),
        )
        .set(user)
        .returning(Self::as_select())
        .get_result::<Self>(conn)?;
        Ok(updated_user)
    }

    pub fn soft_delete(
        conn: &mut DbConnection,
        user_id: Uuid,
        authenticated: &Authenticated,
    ) -> Result<(), ApiError> {
        conn.transaction(|conn| {
            let before = users::table
                .find(user_id)
                .for_update()
                .select(Self::as_select())
                .first::<Self>(conn)?;

            if before.account_status == AccountStatus::Deleted {
                return Ok(());
            }

            // only delete non verifications records/submissions

            let verification_submission_ids = records::table
                .filter(records::submitted_by.eq(user_id))
                .filter(records::is_verification.eq(true))
                .select(records::submission_id)
                .load::<Uuid>(conn)?;
            let has_verifications = !verification_submission_ids.is_empty();

            let deleted_records = diesel::delete(
                records::table
                    .filter(records::submitted_by.eq(user_id))
                    .filter(records::is_verification.eq(false)),
            )
                .returning(Record::as_select())
                .get_results::<Record>(conn)?;
            for record in &deleted_records {
                AuditLogEntry::log_delete(conn, Some(authenticated.user_id), record)?;
            }

            diesel::update(submissions::table.filter(submissions::submitted_by.eq(user_id)))
                .filter(submissions::id.ne_all(&verification_submission_ids))
                .filter(submissions::status.ne(SubmissionStatus::Denied))
                .set((
                    submissions::status.eq(SubmissionStatus::Denied),
                    submissions::reviewer_id.eq(Some(authenticated.user_id)),
                    submissions::reviewer_notes.eq("This submission has been automatically rejected because this user has been deleted."),
                ))
                .execute(conn)?;

            diesel::delete(oauth_connected_accounts::table.filter(oauth_connected_accounts::user_id.eq(user_id)))
                .execute(conn)?;
            diesel::delete(oauth_requests::table.filter(oauth_requests::user_id.eq(user_id)))
                .execute(conn)?;

            let deleted_user = diesel::update(users::table.find(user_id))
                .set((
                    (!before.placeholder || !has_verifications)
                        .then_some(users::username.eq(diesel::dsl::sql::<VarChar>("DEFAULT"))),
                    users::global_name.eq(if has_verifications { before.global_name.clone() } else { "Deleted User".to_owned() }),
                    users::account_status.eq(if has_verifications { before.account_status } else { AccountStatus::Deleted }),
                    users::placeholder.eq(before.placeholder || has_verifications),
                    users::discord_id.eq(None::<String>),
                    users::discord_avatar.eq(None::<String>),
                    users::discord_avatar_decoration.eq(None::<String>),
                    users::last_discord_avatar_update.eq(None::<NaiveDateTime>),
                    users::description.eq(None::<String>),
                    users::country.eq(None::<i32>),
                    users::background_level.eq(None::<i32>),
                    users::featured_badge_code.eq(None::<String>),
                    users::access_valid_after.eq(Utc::now()),
                ))
                .returning(Self::as_select())
                .get_result::<Self>(conn)?;
            AuditLogEntry::log_update(conn, Some(authenticated.user_id), &before, &deleted_user)?;
            Ok(())
        })
    }

    pub fn update_account_status(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
        user_id: Uuid,
        account_status: AccountStatus,
    ) -> Result<User, ApiError> {
        if account_status == AccountStatus::Deleted {
            return Err(ApiError::BadRequest(
                "Account status cannot be manually set to deleted, use the delete endpoint instead.",
            ));
        }
        let user = diesel::update(
            users::table
                .filter(users::id.eq(user_id))
                .filter(users::account_status.ne(AccountStatus::Deleted)),
        )
        .set(users::account_status.eq(account_status))
        .returning(Self::as_select())
        .get_result::<Self>(conn)?;

        if account_status >= AccountStatus::ListBanned {
            // Set user's pending submissions to `Denied`
            diesel::update(
                submissions::table
                    .filter(submissions::submitted_by.eq(user_id))
                    .filter(submissions::status.eq(SubmissionStatus::Pending)),
            )
            .set((
                submissions::status.eq(SubmissionStatus::Denied),
                submissions::reviewer_id.eq(Some(authenticated.user_id)),
                submissions::reviewer_notes.eq("This submission has been automatically rejected because this player has been list banned."),
            ))
            .execute(conn)?;
        }
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

impl Auditable for User {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::User;

    fn entity_id(&self) -> Uuid {
        self.id
    }

    fn entity_display(&self, _conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(self.global_name.clone())
    }
}

impl UserResolved {
    pub fn from_user(
        conn: &mut DbConnection,
        user: User,
        authenticated: Option<&Authenticated>,
    ) -> Result<Self, ApiError> {
        let clan = clans::table
            .inner_join(clan_members::table)
            .filter(clan_members::user_id.eq(user.id))
            .select(Clan::as_select())
            .first::<Clan>(conn)
            .optional()?;

        let mut roles = user_roles::table
            .inner_join(roles::table)
            .filter(user_roles::user_id.eq(user.id))
            .select(Role::as_select())
            .load::<Role>(conn)?;

        let can_view_hidden_roles = match authenticated.as_ref() {
            Some(auth) if auth.user_id == user.id => true,
            Some(auth) => auth.has_permission(conn, Permission::RoleAssign)?,
            None => false,
        };

        if !can_view_hidden_roles {
            roles.retain(|role| !role.hide);
        }

        let scopes = permission::get_user_permissions(conn, user.id, !can_view_hidden_roles)?;

        let badges = UserBadge::find_all(conn, user.id)?;

        Ok(UserResolved {
            user,
            clan,
            roles,
            scopes,
            badges,
        })
    }
}
