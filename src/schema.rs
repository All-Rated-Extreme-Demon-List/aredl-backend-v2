// @generated automatically by Diesel CLI.

diesel::table! {
    aredl_levels (id) {
        id -> Uuid,
        position -> Int4,
        name -> Varchar,
        points -> Int4,
        legacy -> Bool,
        level_id -> Int4,
        two_player -> Bool,
    }
}

diesel::table! {
    aredl_position_history (i) {
        i -> Int4,
        new_position -> Nullable<Int4>,
        old_position -> Nullable<Int4>,
        legacy -> Nullable<Bool>,
        affected_level -> Uuid,
        created_at -> Nullable<Timestamp>,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        user_name -> Varchar,
        global_name -> Varchar,
        placeholder -> Bool,
    }
}

diesel::joinable!(aredl_position_history -> aredl_levels (affected_level));

diesel::allow_tables_to_appear_in_same_query!(
    aredl_levels,
    aredl_position_history,
    users,
);
