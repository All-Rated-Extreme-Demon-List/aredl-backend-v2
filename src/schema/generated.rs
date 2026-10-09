// @generated automatically by Diesel CLI.

pub mod public {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "bounty_difficulty"))]
        pub struct BountyDifficulty;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "bounty_type"))]
        pub struct BountyType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "custom_id_status"))]
        pub struct CustomIdStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "custom_id_type"))]
        pub struct CustomIdType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "level_notes_type"))]
        pub struct LevelNotesType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "level_status"))]
        pub struct LevelStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "level_update_type"))]
        pub struct LevelUpdateType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "notification_type"))]
        pub struct NotificationType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "oauth_provider"))]
        pub struct OauthProvider;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "shift_status"))]
        pub struct ShiftStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "submission_status"))]
        pub struct SubmissionStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "weekday"))]
        pub struct Weekday;
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::BountyType;
        use super::sql_types::BountyDifficulty;

        bounties (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            bounty_type -> BountyType,
            bounty_difficulty -> BountyDifficulty,
            start_date -> Timestamptz,
            end_date -> Nullable<Timestamptz>,
            target_submissions -> Nullable<Int4>,
            is_target_public -> Bool,
        }
    }

    diesel::table! {
        bounty_completed (list_id, user_id, bounty_id) {
            list_id -> Int2,
            user_id -> Uuid,
            bounty_id -> Uuid,
            completed_at -> Timestamptz,
        }
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
        last_gddl_update (id) {
            list_id -> Int2,
            id -> Uuid,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CustomIdType;
        use super::sql_types::CustomIdStatus;

        level_custom_copies (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            copy_id -> Int4,
            added_by -> Uuid,
            description -> Nullable<Varchar>,
            created_at -> Timestamptz,
            id_type -> CustomIdType,
            status -> CustomIdStatus,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::LevelNotesType;

        level_notes (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            note -> Text,
            note_type -> LevelNotesType,
            timestamp -> Nullable<Timestamptz>,
            added_by -> Uuid,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::LevelUpdateType;

        level_updates (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            changelog -> Nullable<Text>,
            update_type -> LevelUpdateType,
            timestamp -> Timestamptz,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::LevelStatus;

        levels (id) {
            list_id -> Int2,
            id -> Uuid,
            position -> Nullable<Int4>,
            name -> Varchar,
            publisher_id -> Uuid,
            points -> Int4,
            level_id -> Int4,
            two_player -> Bool,
            tags -> Array<Nullable<Text>>,
            description -> Nullable<Varchar>,
            song -> Nullable<Int4>,
            edel_enjoyment -> Nullable<Float8>,
            is_edel_pending -> Bool,
            gddl_tier -> Nullable<Float8>,
            nlw_tier -> Nullable<Varchar>,
            status -> LevelStatus,
            requires_raw_footage -> Bool,
            nlw_tier_estimate -> Nullable<Varchar>,
        }
    }

    diesel::table! {
        levels_created (list_id, level_id, user_id) {
            list_id -> Int2,
            level_id -> Uuid,
            user_id -> Uuid,
        }
    }

    diesel::table! {
        lists (id) {
            id -> Int2,
            name -> Text,
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
        use diesel::sql_types::*;
        use super::sql_types::OauthProvider;

        oauth_connected_accounts (id) {
            id -> Uuid,
            user_id -> Uuid,
            provider -> OauthProvider,
            provider_user_id -> Text,
            provider_user_name -> Nullable<Text>,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::OauthProvider;

        oauth_requests (csrf_state) {
            csrf_state -> Varchar,
            pkce_verifier -> Nullable<Varchar>,
            callback -> Nullable<Varchar>,
            created_at -> Nullable<Timestamptz>,
            provider -> OauthProvider,
            user_id -> Nullable<Uuid>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::OauthProvider;

        oauth_tokens (provider) {
            provider -> OauthProvider,
            access_token -> Nullable<Text>,
            refresh_token -> Nullable<Text>,
            expires_at -> Nullable<Timestamptz>,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        pack_levels (list_id, pack_id, level_id) {
            list_id -> Int2,
            pack_id -> Uuid,
            level_id -> Uuid,
        }
    }

    diesel::table! {
        pack_tiers (id) {
            list_id -> Int2,
            id -> Uuid,
            name -> Varchar,
            color -> Varchar,
            placement -> Int4,
        }
    }

    diesel::table! {
        packs (id) {
            list_id -> Int2,
            id -> Uuid,
            name -> Varchar,
            tier -> Uuid,
        }
    }

    diesel::table! {
        permissions (permission) {
            permission -> Varchar,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::LevelStatus;

        position_history (list_id, i) {
            list_id -> Int2,
            i -> Int4,
            new_position -> Nullable<Int4>,
            old_position -> Nullable<Int4>,
            affected_level -> Uuid,
            level_above -> Nullable<Uuid>,
            level_below -> Nullable<Uuid>,
            created_at -> Timestamptz,
            old_status -> Nullable<LevelStatus>,
            new_status -> LevelStatus,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::LevelStatus;

        position_history_full_view (list_id, ord, affected_level) {
            list_id -> Int2,
            ord -> Int4,
            affected_level -> Uuid,
            position -> Nullable<Int4>,
            moved -> Bool,
            status -> LevelStatus,
            action_at -> Timestamptz,
            cause -> Uuid,
            pos_diff -> Nullable<Int4>,
        }
    }

    diesel::table! {
        records (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            submitted_by -> Uuid,
            mobile -> Bool,
            video_url -> Varchar,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_verification -> Bool,
            completion_time -> Nullable<Int8>,
            hide_video -> Bool,
            submission_id -> Uuid,
            achieved_at -> Timestamptz,
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
            #[max_length = 50]
            timezone -> Varchar,
        }
    }

    diesel::table! {
        role_permissions (role_id, permission) {
            role_id -> Int4,
            permission -> Varchar,
        }
    }

    diesel::table! {
        roles (id) {
            id -> Int4,
            privilege_level -> Int4,
            role_desc -> Varchar,
            hide -> Bool,
            inherits_from_role_id -> Nullable<Int4>,
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
        submission_daily_level_stats (list_id, day, level_id) {
            list_id -> Int2,
            day -> Date,
            level_id -> Uuid,
            submitted -> Int8,
            accepted -> Int8,
            denied -> Int8,
            under_consideration -> Int8,
            reviewed -> Int8,
        }
    }

    diesel::table! {
        submission_daily_reviewer_stats (list_id, day, reviewer_id) {
            list_id -> Int2,
            day -> Date,
            reviewer_id -> Uuid,
            accepted -> Int8,
            denied -> Int8,
            under_consideration -> Int8,
            reviewed -> Int8,
        }
    }

    diesel::table! {
        submission_daily_total_stats (list_id, day) {
            list_id -> Int2,
            day -> Date,
            submitted -> Int8,
            accepted -> Int8,
            denied -> Int8,
            under_consideration -> Int8,
            reviewed -> Int8,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        submission_history (id) {
            list_id -> Int2,
            id -> Uuid,
            submission_id -> Uuid,
            reviewer_notes -> Nullable<Text>,
            status -> SubmissionStatus,
            timestamp -> Timestamptz,
            user_notes -> Nullable<Text>,
            reviewer_id -> Nullable<Uuid>,
            mobile -> Nullable<Bool>,
            custom_copy_id -> Nullable<Int4>,
            video_url -> Nullable<Varchar>,
            raw_url -> Nullable<Varchar>,
            mod_menu -> Nullable<Varchar>,
            priority -> Nullable<Bool>,
            private_reviewer_notes -> Nullable<Text>,
            locked -> Nullable<Bool>,
            completion_time -> Nullable<Int8>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        submissions (id) {
            list_id -> Int2,
            id -> Uuid,
            level_id -> Uuid,
            submitted_by -> Uuid,
            mobile -> Bool,
            custom_copy_id -> Nullable<Int4>,
            video_url -> Varchar,
            raw_url -> Nullable<Varchar>,
            reviewer_id -> Nullable<Uuid>,
            priority -> Bool,
            reviewer_notes -> Nullable<Varchar>,
            user_notes -> Nullable<Varchar>,
            created_at -> Timestamptz,
            status -> SubmissionStatus,
            mod_menu -> Nullable<Varchar>,
            updated_at -> Timestamptz,
            completion_time -> Nullable<Int8>,
            private_reviewer_notes -> Nullable<Text>,
            locked -> Bool,
            priority_at -> Timestamptz,
        }
    }

    diesel::table! {
        submissions_enabled (id) {
            list_id -> Int2,
            id -> Uuid,
            enabled -> Bool,
            moderator -> Uuid,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        user_badges (user_id, badge_code) {
            user_id -> Uuid,
            badge_code -> Varchar,
            unlocked_at -> Timestamptz,
            description -> Nullable<Varchar>,
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
            access_valid_after -> Timestamptz,
            created_at -> Timestamptz,
            background_level -> Nullable<Int4>,
            last_discord_avatar_update -> Nullable<Timestamp>,
            featured_badge_code -> Nullable<Varchar>,
            discord_avatar_decoration -> Nullable<Varchar>,
        }
    }

    diesel::joinable!(bounties -> lists (list_id));
    diesel::joinable!(bounty_completed -> lists (list_id));
    diesel::joinable!(bounty_completed -> users (user_id));
    diesel::joinable!(clan_invites -> clans (clan_id));
    diesel::joinable!(clan_members -> clans (clan_id));
    diesel::joinable!(clan_members -> users (user_id));
    diesel::joinable!(last_gddl_update -> lists (list_id));
    diesel::joinable!(level_custom_copies -> lists (list_id));
    diesel::joinable!(level_custom_copies -> users (added_by));
    diesel::joinable!(level_notes -> lists (list_id));
    diesel::joinable!(level_notes -> users (added_by));
    diesel::joinable!(level_updates -> lists (list_id));
    diesel::joinable!(levels -> lists (list_id));
    diesel::joinable!(levels -> users (publisher_id));
    diesel::joinable!(levels_created -> lists (list_id));
    diesel::joinable!(levels_created -> users (user_id));
    diesel::joinable!(merge_logs -> users (primary_user));
    diesel::joinable!(notifications -> users (user_id));
    diesel::joinable!(oauth_connected_accounts -> users (user_id));
    diesel::joinable!(oauth_requests -> users (user_id));
    diesel::joinable!(pack_levels -> lists (list_id));
    diesel::joinable!(pack_tiers -> lists (list_id));
    diesel::joinable!(packs -> lists (list_id));
    diesel::joinable!(position_history -> lists (list_id));
    diesel::joinable!(position_history_full_view -> lists (list_id));
    diesel::joinable!(records -> lists (list_id));
    diesel::joinable!(records -> users (submitted_by));
    diesel::joinable!(recurrent_shifts -> users (user_id));
    diesel::joinable!(role_permissions -> permissions (permission));
    diesel::joinable!(role_permissions -> roles (role_id));
    diesel::joinable!(shifts -> users (user_id));
    diesel::joinable!(submission_daily_level_stats -> lists (list_id));
    diesel::joinable!(submission_daily_reviewer_stats -> lists (list_id));
    diesel::joinable!(submission_daily_reviewer_stats -> users (reviewer_id));
    diesel::joinable!(submission_daily_total_stats -> lists (list_id));
    diesel::joinable!(submission_history -> lists (list_id));
    diesel::joinable!(submission_history -> users (reviewer_id));
    diesel::joinable!(submissions -> lists (list_id));
    diesel::joinable!(submissions_enabled -> lists (list_id));
    diesel::joinable!(submissions_enabled -> users (moderator));
    diesel::joinable!(user_badges -> users (user_id));
    diesel::joinable!(user_roles -> roles (role_id));
    diesel::joinable!(user_roles -> users (user_id));

    diesel::allow_tables_to_appear_in_same_query!(
        bounties,
        bounty_completed,
        clan_invites,
        clan_members,
        clans,
        last_gddl_update,
        level_custom_copies,
        level_notes,
        level_updates,
        levels,
        levels_created,
        lists,
        matview_refresh_log,
        merge_logs,
        merge_requests,
        notifications,
        oauth_connected_accounts,
        oauth_requests,
        oauth_tokens,
        pack_levels,
        pack_tiers,
        packs,
        permissions,
        position_history,
        position_history_full_view,
        records,
        recurrent_shifts,
        role_permissions,
        roles,
        shifts,
        submission_daily_level_stats,
        submission_daily_reviewer_stats,
        submission_daily_total_stats,
        submission_history,
        submissions,
        submissions_enabled,
        user_badges,
        user_roles,
        users,
    );
}
