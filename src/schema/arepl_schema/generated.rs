// @generated automatically by Diesel CLI.

pub mod arepl {
    pub mod sql_types {
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
        arepl.last_gddl_update (id) {
            id -> Uuid,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        arepl.levels (id) {
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
        arepl.levels_created (level_id, user_id) {
            level_id -> Uuid,
            user_id -> Uuid,
        }
    }

    diesel::table! {
        arepl.pack_levels (pack_id, level_id) {
            pack_id -> Uuid,
            level_id -> Uuid,
        }
    }

    diesel::table! {
        arepl.pack_tiers (id) {
            id -> Uuid,
            name -> Varchar,
            color -> Varchar,
            placement -> Int4,
        }
    }

    diesel::table! {
        arepl.packs (id) {
            id -> Uuid,
            name -> Varchar,
            tier -> Uuid,
        }
    }

    diesel::table! {
        arepl.position_history (i) {
            i -> Int4,
            new_position -> Nullable<Int4>,
            old_position -> Nullable<Int4>,
            legacy -> Nullable<Bool>,
            affected_level -> Uuid,
            level_above -> Nullable<Uuid>,
            level_below -> Nullable<Uuid>,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        arepl.records (id) {
            id -> Uuid,
            level_id -> Uuid,
            submitted_by -> Uuid,
            mobile -> Bool,
            ldm_id -> Nullable<Int4>,
            video_url -> Varchar,
            raw_url -> Nullable<Varchar>,
            placement_order -> Int4,
            reviewer_id -> Nullable<Uuid>,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_verification -> Bool,
            reviewer_notes -> Nullable<Varchar>,
            mod_menu -> Nullable<Varchar>,
            user_notes -> Nullable<Varchar>,
            completion_time -> Int8,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::Weekday;

        arepl.recurrent_shifts (id) {
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
        use diesel::sql_types::*;
        use super::sql_types::ShiftStatus;

        arepl.shifts (id) {
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
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        arepl.submission_history (id) {
            id -> Uuid,
            submission_id -> Uuid,
            record_id -> Nullable<Uuid>,
            reviewer_notes -> Nullable<Text>,
            status -> SubmissionStatus,
            timestamp -> Timestamptz,
            user_notes -> Nullable<Text>,
            reviewer_id -> Nullable<Uuid>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        arepl.submissions (id) {
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
            created_at -> Timestamptz,
            status -> SubmissionStatus,
            mod_menu -> Nullable<Varchar>,
            updated_at -> Timestamptz,
            completion_time -> Int8,
        }
    }

    diesel::allow_tables_to_appear_in_same_query!(
        last_gddl_update,
        levels,
        levels_created,
        pack_levels,
        pack_tiers,
        packs,
        position_history,
        records,
        recurrent_shifts,
        shifts,
        submission_history,
        submissions,
    );
}
