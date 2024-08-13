// @generated automatically by Diesel CLI.

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
        idl_enjoyment -> Nullable<Float8>,
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
    }
}

diesel::table! {
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
        is_update -> Bool,
        is_rejected -> Bool,
        rejection_reason -> Nullable<Varchar>,
        additional_notes -> Nullable<Varchar>,
        created_at -> Timestamp,
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
    user_roles (role_id, user_id) {
        role_id -> Int4,
        user_id -> Uuid,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        username -> Varchar,
        global_name -> Varchar,
        discord_id -> Nullable<Varchar>,
        placeholder -> Bool,
        country -> Nullable<Int4>,
        discord_avatar -> Nullable<Varchar>,
        discord_banner -> Nullable<Varchar>,
        discord_accent_color -> Nullable<Int4>,
    }
}

diesel::joinable!(aredl_levels -> users (publisher_id));
diesel::joinable!(aredl_levels_created -> aredl_levels (level_id));
diesel::joinable!(aredl_levels_created -> users (user_id));
diesel::joinable!(aredl_pack_levels -> aredl_levels (level_id));
diesel::joinable!(aredl_pack_levels -> aredl_packs (pack_id));
diesel::joinable!(aredl_packs -> aredl_pack_tiers (tier));
diesel::joinable!(aredl_records -> aredl_levels (level_id));
diesel::joinable!(aredl_submissions -> aredl_levels (level_id));
diesel::joinable!(user_roles -> roles (role_id));
diesel::joinable!(user_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_levels,
    aredl_levels_created,
    aredl_pack_levels,
    aredl_pack_tiers,
    aredl_packs,
    aredl_position_history,
    aredl_records,
    aredl_submissions,
    permissions,
    roles,
    user_roles,
    users,
);
