// @generated automatically by Diesel CLI.

pub mod aredl {
    pub mod sql_types {
        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "custom_id_status", schema = "aredl"))]
        pub struct CustomIdStatus;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "custom_id_type", schema = "aredl"))]
        pub struct CustomIdType;

        #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
        #[diesel(postgres_type(name = "submission_status"))]
        pub struct SubmissionStatus;
    }

    diesel::table! {
        aredl.guideline_updates (id) {
            id -> Uuid,
            moderator -> Uuid,
            text -> Varchar,
            created_at -> Timestamptz,
        }
    }

    diesel::table! {
        aredl.last_gddl_update (id) {
            id -> Uuid,
            updated_at -> Timestamptz,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::CustomIdType;
        use super::sql_types::CustomIdStatus;

        aredl.level_ldms (id) {
            id -> Uuid,
            level_id -> Uuid,
            ldm_id -> Int4,
            added_by -> Uuid,
            description -> Nullable<Varchar>,
            created_at -> Timestamptz,
            id_type -> CustomIdType,
            status -> CustomIdStatus,
        }
    }

    diesel::table! {
        aredl.levels (id) {
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
        aredl.levels_created (level_id, user_id) {
            level_id -> Uuid,
            user_id -> Uuid,
        }
    }

    diesel::table! {
        aredl.pack_levels (pack_id, level_id) {
            pack_id -> Uuid,
            level_id -> Uuid,
        }
    }

    diesel::table! {
        aredl.pack_tiers (id) {
            id -> Uuid,
            name -> Varchar,
            color -> Varchar,
            placement -> Int4,
        }
    }

    diesel::table! {
        aredl.packs (id) {
            id -> Uuid,
            name -> Varchar,
            tier -> Uuid,
        }
    }

    diesel::table! {
        aredl.position_history (i) {
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
        aredl.records (id) {
            id -> Uuid,
            level_id -> Uuid,
            submitted_by -> Uuid,
            mobile -> Bool,
            video_url -> Varchar,
            created_at -> Timestamptz,
            updated_at -> Timestamptz,
            is_verification -> Bool,
            hide_video -> Bool,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        aredl.submission_history (id) {
            id -> Uuid,
            submission_id -> Uuid,
            reviewer_notes -> Nullable<Text>,
            status -> SubmissionStatus,
            timestamp -> Timestamptz,
            user_notes -> Nullable<Text>,
            reviewer_id -> Nullable<Uuid>,
            mobile -> Nullable<Bool>,
            ldm_id -> Nullable<Int4>,
            video_url -> Nullable<Varchar>,
            raw_url -> Nullable<Varchar>,
            mod_menu -> Nullable<Varchar>,
            priority -> Nullable<Bool>,
            private_reviewer_notes -> Nullable<Text>,
        }
    }

    diesel::table! {
        use diesel::sql_types::*;
        use super::sql_types::SubmissionStatus;

        aredl.submissions (id) {
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
            private_reviewer_notes -> Nullable<Text>,
        }
    }

    diesel::table! {
        aredl.submissions_enabled (id) {
            id -> Uuid,
            enabled -> Bool,
            moderator -> Uuid,
            created_at -> Timestamptz,
        }
    }

    diesel::joinable!(level_ldms -> levels (level_id));
    diesel::joinable!(submission_history -> submissions (submission_id));

    diesel::allow_tables_to_appear_in_same_query!(
        guideline_updates,
        last_gddl_update,
        level_ldms,
        levels,
        levels_created,
        pack_levels,
        pack_tiers,
        packs,
        position_history,
        records,
        submission_history,
        submissions,
        submissions_enabled,
    );
}
