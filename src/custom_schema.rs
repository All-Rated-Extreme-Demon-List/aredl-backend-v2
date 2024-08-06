use crate::schema::aredl_levels;
use crate::schema::aredl_pack_tiers;
use crate::schema::aredl_pack_levels;

diesel::table! {
    aredl_position_history_full_view (affected_level) {
        affected_level -> Uuid,
        position -> Nullable<Int4>,
        moved -> Bool,
        legacy -> Bool,
        action_at -> Timestamp,
        cause -> Uuid,
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