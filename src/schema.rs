// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "notification_type"))]
    pub struct NotificationType;

    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "submission_status"))]
    pub struct SubmissionStatus;
}

diesel::table! {
    aredl_last_gddl_update (id) {
        id -> Uuid,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    aredl_levels (id) {
        id -> Uuid,
        position -> Int4,
        name -> Varchar,
        publisher_id -> Uuid,
        points -> Int4,
        legacy -> Bool,
        level_id -> Int4,
        two_player -> Bool,
        tags -> Array<Nullable<Text>>,
        description -> Nullable<Varchar>,
        song -> Nullable<Int4>,
        edel_enjoyment -> Nullable<Float8>,
        is_edel_pending -> Bool,
        gddl_tier -> Nullable<Float8>,
        nlw_tier -> Nullable<Varchar>,
    }
}

diesel::table! {
    aredl_levels_created (level_id, user_id) {
        level_id -> Uuid,
        user_id -> Uuid,
    }
}

diesel::table! {
    aredl_pack_levels (pack_id, level_id) {
        pack_id -> Uuid,
        level_id -> Uuid,
    }
}

diesel::table! {
    aredl_pack_tiers (id) {
        id -> Uuid,
        name -> Varchar,
        color -> Varchar,
        placement -> Int4,
    }
}

diesel::table! {
    aredl_packs (id) {
        id -> Uuid,
        name -> Varchar,
        tier -> Uuid,
    }
}

diesel::table! {
    aredl_position_history (i) {
        i -> Int4,
        new_position -> Nullable<Int4>,
        old_position -> Nullable<Int4>,
        legacy -> Nullable<Bool>,
        affected_level -> Uuid,
        level_above -> Nullable<Uuid>,
        level_below -> Nullable<Uuid>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    aredl_records (id) {
        id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        ldm_id -> Nullable<Int4>,
        video_url -> Varchar,
        raw_url -> Nullable<Varchar>,
        placement_order -> Int4,
        reviewer_id -> Nullable<Uuid>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
        is_verification -> Bool,
        reviewer_notes -> Nullable<Varchar>,
        mod_menu -> Nullable<Varchar>,
        user_notes -> Nullable<Varchar>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SubmissionStatus;

    aredl_submissions (id) {
        id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        ldm_id -> Nullable<Int4>,
        video_url -> Varchar,
        raw_url -> Nullable<Varchar>,
        reviewer_id -> Nullable<Uuid>,
        priority -> Bool,
        reviewer_notes -> Nullable<Varchar>,
        user_notes -> Nullable<Varchar>,
        created_at -> Timestamp,
        status -> SubmissionStatus,
        mod_menu -> Nullable<Varchar>,
    }
}

diesel::table! {
    clan_invites (id) {
        id -> Uuid,
        clan_id -> Uuid,
        user_id -> Uuid,
        invited_by -> Uuid,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    clan_members (id) {
        id -> Uuid,
        clan_id -> Uuid,
        user_id -> Uuid,
        role -> Int4,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    clans (id) {
        id -> Uuid,
        global_name -> Varchar,
        tag -> Varchar,
        description -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        merged_at -> Timestamp,
    }
}

diesel::table! {
    merge_requests (id) {
        id -> Uuid,
        primary_user -> Uuid,
        secondary_user -> Uuid,
        is_rejected -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
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
        created_at -> Timestamp,
    }
}

diesel::table! {
    oauth_requests (csrf_state) {
        csrf_state -> Varchar,
        pkce_verifier -> Varchar,
        nonce -> Varchar,
        callback -> Nullable<Varchar>,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    permissions (permission) {
        privilege_level -> Int4,
        permission -> Varchar,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        privilege_level -> Int4,
        role_desc -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::SubmissionStatus;

    submission_history (id) {
        id -> Uuid,
        submission_id -> Nullable<Uuid>,
        record_id -> Nullable<Uuid>,
        rejection_reason -> Nullable<Text>,
        status -> SubmissionStatus,
        timestamp -> Timestamp,
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
        last_country_update -> Timestamp,
        ban_level -> Int4,
        discord_avatar -> Nullable<Varchar>,
        discord_banner -> Nullable<Varchar>,
        discord_accent_color -> Nullable<Int4>,
        access_valid_after -> Timestamp,
        created_at -> Timestamp,
    }
}

diesel::joinable!(aredl_last_gddl_update -> aredl_levels (id));
diesel::joinable!(aredl_levels -> users (publisher_id));
diesel::joinable!(aredl_levels_created -> aredl_levels (level_id));
diesel::joinable!(aredl_levels_created -> users (user_id));
diesel::joinable!(aredl_pack_levels -> aredl_levels (level_id));
diesel::joinable!(aredl_pack_levels -> aredl_packs (pack_id));
diesel::joinable!(aredl_packs -> aredl_pack_tiers (tier));
diesel::joinable!(aredl_records -> aredl_levels (level_id));
diesel::joinable!(aredl_submissions -> aredl_levels (level_id));
diesel::joinable!(clan_invites -> clans (clan_id));
diesel::joinable!(clan_members -> clans (clan_id));
diesel::joinable!(clan_members -> users (user_id));
diesel::joinable!(merge_logs -> users (primary_user));
diesel::joinable!(notifications -> users (user_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_last_gddl_update,
    aredl_levels,
    aredl_levels_created,
    aredl_pack_levels,
    aredl_pack_tiers,
    aredl_packs,
    aredl_position_history,
    aredl_records,
    aredl_submissions,
    clan_invites,
    clan_members,
    clans,
    merge_logs,
    merge_requests,
    notifications,
    oauth_requests,
    permissions,
    roles,
    submission_history,
    user_roles,
    users,
);
