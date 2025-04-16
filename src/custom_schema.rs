use crate::schema::{aredl_levels, aredl_packs, users, aredl_pack_tiers, aredl_pack_levels, clans};

diesel::table! {
    aredl_position_history_full_view (affected_level) {
        ord -> Int4,
        affected_level -> Uuid,
        position -> Nullable<Int4>,
        moved -> Bool,
        legacy -> Bool,
        action_at -> Timestamp,
        cause -> Uuid,
        pos_diff -> Nullable<Int4>,
    }
}

diesel::joinable!(aredl_position_history_full_view -> aredl_levels (affected_level));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_levels,
    aredl_position_history_full_view,
);

diesel::table! {
    aredl_packs_points (id) {
        id -> Uuid,
        name -> Varchar,
        tier -> Uuid,
        points -> Int4,
    }
}

diesel::joinable!(aredl_packs_points -> aredl_pack_tiers (tier));
diesel::joinable!(aredl_pack_levels -> aredl_packs_points (pack_id));

diesel::table! {
    aredl_user_leaderboard (user_id) {
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
    aredl_country_leaderboard (country) {
        rank -> Int4,
        extremes_rank -> Int4,
        country -> Int4,
        level_points -> Int4,
		members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::joinable!(aredl_user_leaderboard -> users (user_id));
diesel::joinable!(aredl_user_leaderboard -> aredl_levels (hardest));
diesel::joinable!(aredl_user_leaderboard -> clans (clan_id));
diesel::joinable!(aredl_country_leaderboard -> aredl_levels (hardest));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_user_leaderboard,
    aredl_levels,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_country_leaderboard,
    aredl_levels,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_user_leaderboard,
    users,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_user_leaderboard,
    clans,
);

diesel::table! {
    aredl_completed_packs (user_id) {
        user_id -> Uuid,
        pack_id -> Uuid,
    }
}

diesel::joinable!(aredl_completed_packs -> users (user_id));
diesel::joinable!(aredl_completed_packs -> aredl_packs (pack_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_completed_packs,
    aredl_packs,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_completed_packs,
    aredl_pack_tiers,
);

diesel::table! {
    aredl_min_placement_country_records (id) {
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
        country -> Int4,
        order_pos -> Int4,
    }
}

diesel::joinable!(aredl_min_placement_country_records -> users (submitted_by));
diesel::joinable!(aredl_min_placement_country_records -> aredl_levels (level_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_min_placement_country_records,
    users,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_min_placement_country_records,
    aredl_levels,
);


diesel::table! {
    aredl_clans_leaderboard (clan_id) {
        rank -> Int4,
        extremes_rank -> Int4,
        clan_id -> Uuid,
        level_points -> Int4,
		members_count -> Int4,
        hardest -> Nullable<Uuid>,
        extremes -> Int4
    }
}

diesel::joinable!(aredl_clans_leaderboard -> aredl_levels (hardest));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_clans_leaderboard,
    aredl_levels,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_clans_leaderboard,
    clans,
);

diesel::table! {
    aredl_min_placement_clans_records (id) {
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
        clan_id -> Uuid,
        order_pos -> Int4,
    }
}

diesel::joinable!(aredl_min_placement_clans_records -> users (submitted_by));
diesel::joinable!(aredl_min_placement_clans_records -> aredl_levels (level_id));
diesel::joinable!(aredl_min_placement_clans_records -> clans (clan_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_min_placement_clans_records,
    users,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_min_placement_clans_records,
    aredl_levels,
);

diesel::allow_tables_to_appear_in_same_query!(
    aredl_min_placement_clans_records,
    clans,
);