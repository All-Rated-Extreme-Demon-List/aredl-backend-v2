// this should be used to declare things like views, which
// diesel won't autogenerate in schema.rs

use crate::schema::arepl::{
    levels, levels_created, pack_levels, pack_tiers, packs, records, recurrent_shifts, shifts,
    submission_history, submissions, submissions_enabled, level_ldms
};
use crate::schema::{clan_members, clans, users};

// Fixing diesel missing public schema joins

diesel::joinable!(levels -> users (publisher_id));
diesel::joinable!(levels_created -> users (user_id));
diesel::joinable!(records -> users (submitted_by));
diesel::joinable!(recurrent_shifts -> users (user_id));
diesel::joinable!(shifts -> users (user_id));
diesel::joinable!(submission_history -> users (reviewer_id));

diesel::allow_tables_to_appear_in_same_query!(levels, users);
diesel::allow_tables_to_appear_in_same_query!(levels, clans);
diesel::allow_tables_to_appear_in_same_query!(levels, clan_members);
diesel::allow_tables_to_appear_in_same_query!(levels_created, users);
diesel::allow_tables_to_appear_in_same_query!(levels_created, clans);
diesel::allow_tables_to_appear_in_same_query!(levels_created, clan_members);
diesel::allow_tables_to_appear_in_same_query!(records, users);
diesel::allow_tables_to_appear_in_same_query!(records, clans);
diesel::allow_tables_to_appear_in_same_query!(records, clan_members);
diesel::allow_tables_to_appear_in_same_query!(recurrent_shifts, users);
diesel::allow_tables_to_appear_in_same_query!(shifts, users);
diesel::allow_tables_to_appear_in_same_query!(submission_history, users);

diesel::table! {
    arepl.position_history_full_view (affected_level) {
        ord -> Int4,
        affected_level -> Uuid,
        position -> Nullable<Int4>,
        moved -> Bool,
        legacy -> Bool,
        action_at -> Timestamptz,
        cause -> Uuid,
        pos_diff -> Nullable<Int4>,
    }
}

diesel::joinable!(position_history_full_view -> levels (affected_level));

diesel::allow_tables_to_appear_in_same_query!(levels, position_history_full_view,);

diesel::table! {
    arepl.packs_points (id) {
        id -> Uuid,
        name -> Varchar,
        tier -> Uuid,
        points -> Int4,
    }
}

diesel::joinable!(packs_points -> pack_tiers (tier));
diesel::joinable!(pack_levels -> packs_points (pack_id));

diesel::table! {
    arepl.user_leaderboard (user_id) {
        rank -> Int4,
        raw_rank -> Int4,
        extremes_rank -> Int4,
        country_rank -> Int4,
        country_raw_rank -> Int4,
        country_extremes_rank -> Int4,
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
    arepl.country_leaderboard (country) {
        rank -> Int4,
        extremes_rank -> Int4,
        country -> Int4,
        level_points -> Int4,
        members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::joinable!(user_leaderboard -> users (user_id));
diesel::joinable!(user_leaderboard -> levels (hardest));
diesel::joinable!(user_leaderboard -> clans (clan_id));
diesel::joinable!(country_leaderboard -> levels (hardest));

diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, levels,);

diesel::allow_tables_to_appear_in_same_query!(country_leaderboard, levels,);

diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, users,);

diesel::allow_tables_to_appear_in_same_query!(user_leaderboard, clans,);

diesel::table! {
    arepl.completed_packs (user_id) {
        user_id -> Uuid,
        pack_id -> Uuid,
    }
}

diesel::joinable!(completed_packs -> users (user_id));
diesel::joinable!(completed_packs -> packs (pack_id));

diesel::allow_tables_to_appear_in_same_query!(completed_packs, packs,);

diesel::allow_tables_to_appear_in_same_query!(completed_packs, pack_tiers,);

diesel::table! {
    arepl.min_placement_country_records (id) {
        id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        ldm_id -> Nullable<Int4>,
        video_url -> Varchar,
        raw_url -> Nullable<Varchar>,
        is_verification -> Bool,
        reviewer_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        country -> Int4,
        order_pos -> Int4,
        completion_time -> Int8,
    }
}

diesel::joinable!(min_placement_country_records -> users (submitted_by));
diesel::joinable!(min_placement_country_records -> levels (level_id));

diesel::allow_tables_to_appear_in_same_query!(min_placement_country_records, users,);

diesel::allow_tables_to_appear_in_same_query!(min_placement_country_records, levels,);

diesel::table! {
    arepl.clans_leaderboard (clan_id) {
        rank -> Int4,
        extremes_rank -> Int4,
        clan_id -> Uuid,
        level_points -> Int4,
        members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::joinable!(clans_leaderboard -> levels (hardest));

diesel::allow_tables_to_appear_in_same_query!(clans_leaderboard, levels,);

diesel::allow_tables_to_appear_in_same_query!(clans_leaderboard, clans,);

diesel::table! {
    arepl.min_placement_clans_records (id) {
        id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        ldm_id -> Nullable<Int4>,
        video_url -> Varchar,
        raw_url -> Nullable<Varchar>,
        is_verification -> Bool,
        reviewer_id -> Nullable<Uuid>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        clan_id -> Uuid,
        order_pos -> Int4,
        completion_time -> Int8,
    }
}

diesel::joinable!(min_placement_clans_records -> users (submitted_by));
diesel::joinable!(min_placement_clans_records -> levels (level_id));
diesel::joinable!(min_placement_clans_records -> clans (clan_id));

diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, users,);

diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, levels,);

diesel::allow_tables_to_appear_in_same_query!(min_placement_clans_records, clans,);

diesel::table! {
    use diesel::sql_types::*;
    use crate::schema::arepl::sql_types::SubmissionStatus;
    arepl.submissions_with_priority (id) {
        id -> Uuid,
        level_id -> Uuid,
        submitted_by -> Uuid,
        mobile -> Bool,
        ldm_id -> Nullable<Int4>,
        video_url -> Varchar,
        raw_url -> Nullable<Varchar>,
        reviewer_id -> Nullable<Uuid>,
        priority -> Bool,
        priority_value -> Bigint,
        reviewer_notes -> Nullable<Varchar>,
        user_notes -> Nullable<Varchar>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        status -> SubmissionStatus,
        mod_menu -> Nullable<Varchar>,
        completion_time -> Int8,
    }
}

diesel::joinable!(submissions -> submissions_with_priority (id));
diesel::joinable!(users -> level_ldms (id));

diesel::allow_tables_to_appear_in_same_query!(submissions, submissions_with_priority);
diesel::allow_tables_to_appear_in_same_query!(levels, submissions_with_priority);
diesel::allow_tables_to_appear_in_same_query!(users, submissions_with_priority);
diesel::allow_tables_to_appear_in_same_query!(users, submissions_enabled);
diesel::allow_tables_to_appear_in_same_query!(users, level_ldms);
