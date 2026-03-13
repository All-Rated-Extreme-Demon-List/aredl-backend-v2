// @generated automatically by Diesel CLI.

pub mod public {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "notification_type"))]
        pub struct NotificationType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "shift_status"))]
        pub struct ShiftStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "weekday"))]
        pub struct Weekday;
    }

    diesel::table! {
        clan_invites (id) {
            id -> Uuid,
            clan_id -> Uuid,
            user_id -> Uuid,
            invited_by -> Uuid,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        clan_members (id) {
            id -> Uuid,
            clan_id -> Uuid,
            user_id -> Uuid,
            role -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        clans (id) {
            id -> Uuid,
            global_name -> Varchar,
            tag -> Varchar,
            description -> Nullable<Text>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        matview_refresh_log (view_name) {
            view_name -> Text,
            last_refresh -> Timestamptz,
        }
    }

    diesel::table! {
        merge_logs (id) {
            id -> Uuid,
            primary_user -> Uuid,
            secondary_user -> Uuid,
            secondary_username -> Varchar,
            secondary_discord_id -> Nullable<Varchar>,
            secondary_global_name -> Varchar,
            merged_at -> Timestamptz,
        }
    }

    diesel::table! {
        merge_requests (id) {
            id -> Uuid,
            primary_user -> Uuid,
            secondary_user -> Uuid,
            is_rejected -> Bool,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_claimed -> Bool,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::NotificationType;

        notifications (id) {
            id -> Uuid,
            user_id -> Uuid,
            content -> Text,
            notification_type -> NotificationType,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        oauth_requests (csrf_state) {
            csrf_state -> Varchar,
            pkce_verifier -> Varchar,
            callback -> Nullable<Varchar>,
            created_at -> Nullable<Timestamptz>,
        }
    }

    diesel::table! {
        permissions (permission) {
            privilege_level -> Int4,
            permission -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Weekday;

        recurrent_shifts (id) {
            id -> Uuid,
            user_id -> Uuid,
            weekday -> Weekday,
            start_hour -> Int4,
            duration -> Int4,
            target_count -> Int4,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        roles (id) {
            id -> Int4,
            privilege_level -> Int4,
            role_desc -> Varchar,
            hide -> Bool,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::ShiftStatus;

        shifts (id) {
            id -> Uuid,
            user_id -> Uuid,
            target_count -> Int4,
            completed_count -> Int4,
            start_at -> Timestamptz,
            end_at -> Timestamptz,
            status -> ShiftStatus,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        user_roles (role_id, user_id) {
            role_id -> Int4,
            user_id -> Uuid,
        }
    }

    diesel::table! {
        users (id) {
            id -> Uuid,
            username -> Varchar,
            json_id -> Nullable<Int8>,
            global_name -> Varchar,
            discord_id -> Nullable<Varchar>,
            placeholder -> Bool,
            description -> Nullable<Text>,
            country -> Nullable<Int4>,
            last_country_update -> Timestamptz,
            ban_level -> Int4,
            discord_avatar -> Nullable<Varchar>,
            discord_banner -> Nullable<Varchar>,
            discord_accent_color -> Nullable<Int4>,
            access_valid_after -> Timestamptz,
            created_at -> Timestamptz,
            background_level -> Int4,
            last_discord_avatar_update -> Nullable<Timestamp>,
        }
    }

    diesel::joinable!(clan_invites -> clans (clan_id));
    diesel::joinable!(clan_members -> clans (clan_id));
    diesel::joinable!(clan_members -> users (user_id));
    diesel::joinable!(merge_logs -> users (primary_user));
    diesel::joinable!(notifications -> users (user_id));
    diesel::joinable!(user_roles -> roles (role_id));
    diesel::joinable!(user_roles -> users (user_id));

    diesel::allow_tables_to_appear_in_same_query!(
        clan_invites,
        clan_members,
        clans,
        matview_refresh_log,
        merge_logs,
        merge_requests,
        notifications,
        oauth_requests,
        permissions,
        recurrent_shifts,
        roles,
        shifts,
        user_roles,
        users,
    );
}
