// this should be used to declare things like views, which
// diesel won't autogenerate in schema.rs

use crate::schema::{
    bounties, bounty_completed, clan_invites, clans, last_gddl_update, level_custom_copies,
    level_notes, level_updates, levels, levels_created, merge_requests, pack_levels, pack_tiers,
    packs, permissions, position_history, position_history_full_view, records, roles,
    submission_daily_level_stats, submission_history, submissions, user_roles, users,
};

diesel::table! {
    role_permissions_full (role_id, permission) {
        role_id -> Int4,
        permission -> Varchar,
    }
}

diesel::table! {
    badge_level_statistics (submitted_by, id) {
        list_id -> Int2,
        submitted_by -> Uuid,
        id -> Uuid,
        name -> Varchar,
        position -> Nullable<Int4>,
        current_position -> Nullable<Int4>,
        level_id -> Int4,
        two_player -> Bool,
        publisher_id -> Uuid,
        edel_enjoyment -> Nullable<Float8>,
        nlw_tier -> Nullable<Varchar>,
        tags -> Array<Nullable<Text>>,
        is_verification -> Bool,
        achieved_at -> Timestamptz,
        is_first_victor -> Bool,
        is_fastest_time -> Bool,
    }
}

diesel::table! {
    user_leaderboard (list_id, user_id) {
        list_id -> Int2,
        rank -> Int4,
        raw_rank -> Int4,
        extremes_rank -> Int4,
        hardest_rank -> Int4,
        country_rank -> Int4,
        country_raw_rank -> Int4,
        country_extremes_rank -> Int4,
        country_hardest_rank -> Int4,
        user_id -> Uuid,
        country -> Nullable<Int4>,
        total_points -> Int4,
        pack_points -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4,
        clan_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    country_leaderboard (list_id, country) {
        list_id -> Int2,
        rank -> Int4,
        extremes_rank -> Int4,
        hardest_rank -> Int4,
        country -> Int4,
        level_points -> Int4,
        members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::table! {
    clans_leaderboard (list_id, clan_id) {
        list_id -> Int2,
        rank -> Int4,
        extremes_rank -> Int4,
        hardest_rank -> Int4,
        clan_id -> Uuid,
        level_points -> Int4,
        members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::table! {
    min_placement_country_records (id) {
        list_id -> Int2,
        id -> Uuid,
        submission_id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        video_url -> Varchar,
        completion_time -> Nullable<Int8>,
        is_verification -> Bool,
        hide_video -> Bool,
        achieved_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        country -> Int4,
        order_pos -> Int4,
        completion_count -> Int8,
    }
}

diesel::table! {
    min_placement_clans_records (id) {
        list_id -> Int2,
        id -> Uuid,
        submission_id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        video_url -> Varchar,
        completion_time -> Nullable<Int8>,
        is_verification -> Bool,
        hide_video -> Bool,
        achieved_at -> Timestamptz,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        clan_id -> Uuid,
        order_pos -> Int4,
        completion_count -> Int8,
    }
}

diesel::table! {
    country_created_levels (list_id, country, level_id, creator_id) {
        list_id -> Int2,
        country -> Int4,
        level_id -> Uuid,
        creator_id -> Uuid,
        order_pos -> Nullable<Int4>,
    }
}

diesel::table! {
    clans_created_levels (list_id, clan_id, level_id, creator_id) {
        list_id -> Int2,
        clan_id -> Uuid,
        level_id -> Uuid,
        creator_id -> Uuid,
        order_pos -> Nullable<Int4>,
    }
}

diesel::table! {
    completed_packs (list_id, user_id, pack_id) {
        list_id -> Int2,
        user_id -> Uuid,
        pack_id -> Uuid,
        completed_at -> Timestamptz,
    }
}

diesel::table! {
    packs_points (id) {
        list_id -> Int2,
        id -> Uuid,
        name -> Varchar,
        tier -> Uuid,
        points -> Int4,
    }
}

diesel::table! {
    clan_member_points (list_id, clan_id, submitted_by) {
        list_id -> Int2,
        clan_id -> Uuid,
        submitted_by -> Uuid,
        completed_levels -> Int8,
        contributed_points -> Float8,
    }
}

diesel::table! {
    record_totals (list_id, level_id) {
        list_id -> Int2,
        level_id -> Nullable<Uuid>,
        records -> Int8,
        verifications -> Int8,
    }
}

diesel::table! {
    submission_totals (list_id, level_id) {
        list_id -> Int2,
        level_id -> Nullable<Uuid>,
        submissions -> Int8,
        percent_of_queue -> Float8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, roles,);
diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, permissions,);
diesel::allow_tables_to_appear_in_same_query!(role_permissions_full, user_roles);
diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, levels,);
diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, users,);
diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, clans,);
diesel::allow_tables_to_appear_in_same_query!(country_leaderboard, levels,);
diesel::allow_tables_to_appear_in_same_query!(clans_leaderboard, levels,);
diesel::allow_tables_to_appear_in_same_query!(clans_leaderboard, clans,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_country_records, users,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_country_records, submissions,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_country_records, levels,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, users,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, submissions,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, levels,);
diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, clans,);
diesel::allow_tables_to_appear_in_same_query!(country_created_levels, levels,);
diesel::allow_tables_to_appear_in_same_query!(country_created_levels, users,);
diesel::allow_tables_to_appear_in_same_query!(clans_created_levels, levels,);
diesel::allow_tables_to_appear_in_same_query!(clans_created_levels, clans,);
diesel::allow_tables_to_appear_in_same_query!(clans_created_levels, users,);
diesel::allow_tables_to_appear_in_same_query!(completed_packs, users,);
diesel::allow_tables_to_appear_in_same_query!(completed_packs, packs,);
diesel::allow_tables_to_appear_in_same_query!(completed_packs, pack_tiers,);
diesel::allow_tables_to_appear_in_same_query!(clan_member_points, clans,);
diesel::allow_tables_to_appear_in_same_query!(clan_member_points, users,);
diesel::allow_tables_to_appear_in_same_query!(record_totals, levels);
diesel::allow_tables_to_appear_in_same_query!(submission_totals, levels);

diesel::joinable!(role_permissions_full -> roles (role_id));
diesel::joinable!(role_permissions_full -> permissions (permission));
diesel::joinable!(user_leaderboard -> users (user_id));
diesel::joinable!(user_leaderboard -> levels (hardest));
diesel::joinable!(user_leaderboard -> clans (clan_id));
diesel::joinable!(country_leaderboard -> levels (hardest));
diesel::joinable!(clans_leaderboard -> levels (hardest));
diesel::joinable!(clans_leaderboard -> clans (clan_id));
diesel::joinable!(min_placement_country_records -> users (submitted_by));
diesel::joinable!(min_placement_country_records -> submissions (submission_id));
diesel::joinable!(min_placement_country_records -> levels (level_id));
diesel::joinable!(min_placement_clans_records -> users (submitted_by));
diesel::joinable!(min_placement_clans_records -> submissions (submission_id));
diesel::joinable!(min_placement_clans_records -> levels (level_id));
diesel::joinable!(min_placement_clans_records -> clans (clan_id));
diesel::joinable!(country_created_levels -> levels (level_id));
diesel::joinable!(country_created_levels -> users (creator_id));
diesel::joinable!(clans_created_levels -> clans (clan_id));
diesel::joinable!(clans_created_levels -> levels (level_id));
diesel::joinable!(clans_created_levels -> users (creator_id));
diesel::joinable!(completed_packs -> users (user_id));
diesel::joinable!(completed_packs -> packs (pack_id));
diesel::joinable!(packs_points -> pack_tiers (tier));
diesel::joinable!(pack_levels -> packs_points (pack_id));
diesel::joinable!(clan_member_points -> clans (clan_id));
diesel::joinable!(clan_member_points -> users (submitted_by));
diesel::joinable!(record_totals -> levels (level_id));
diesel::joinable!(submission_totals -> levels (level_id));

// allows for direct joins that diesel doesn't auto generate because the fkey is composite

diesel::joinable!(position_history -> levels (affected_level));
diesel::joinable!(position_history_full_view -> levels (affected_level));
diesel::joinable!(clan_invites -> users (user_id));
diesel::joinable!(merge_requests -> users (primary_user));
diesel::joinable!(bounties -> levels (level_id));
diesel::joinable!(bounty_completed -> bounties (bounty_id));
diesel::joinable!(last_gddl_update -> levels (id));
diesel::joinable!(level_custom_copies -> levels (level_id));
diesel::joinable!(level_notes -> levels (level_id));
diesel::joinable!(level_updates -> levels (level_id));
diesel::joinable!(levels_created -> levels (level_id));
diesel::joinable!(pack_levels -> levels (level_id));
diesel::joinable!(pack_levels -> packs (pack_id));
diesel::joinable!(packs -> pack_tiers (tier));
diesel::joinable!(records -> levels (level_id));
diesel::joinable!(records -> submissions (submission_id));
diesel::joinable!(submission_daily_level_stats -> levels (level_id));
diesel::joinable!(submission_history -> submissions (submission_id));
diesel::joinable!(submissions -> levels (level_id));
diesel::joinable!(submissions -> users (submitted_by));
