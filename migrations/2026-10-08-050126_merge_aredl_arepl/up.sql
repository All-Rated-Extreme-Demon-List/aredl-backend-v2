SET LOCAL search_path = public, pg_catalog;
SET LOCAL work_mem = '64MB';

CREATE TABLE lists (
    id SMALLINT PRIMARY KEY CHECK (id IN (1, 2)),
    name TEXT NOT NULL UNIQUE
);
INSERT INTO lists VALUES (1, 'classic'), (2, 'platformer');

CREATE TYPE "custom_id_status" AS ENUM ('Published', 'Allowed', 'Banned');
CREATE TYPE "custom_id_type" AS ENUM ('Bugfix', 'GlobedCopy', 'Ldm', 'Other');
CREATE TYPE "level_notes_type" AS ENUM ('ReviewerNotes', 'PublicNotes', 'Other');
CREATE TYPE "level_update_type" AS ENUM ('Buff', 'Nerf', 'Balance', 'BugFix', 'Other');

CREATE SEQUENCE position_history_i_seq AS INTEGER;

CREATE TABLE bounties (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "level_id" uuid NOT NULL,
    "bounty_type" bounty_type NOT NULL,
    "bounty_difficulty" bounty_difficulty NOT NULL,
    "start_date" timestamp with time zone NOT NULL,
    "end_date" timestamp with time zone,
    "target_submissions" integer,
    "is_target_public" boolean NOT NULL DEFAULT false
);

CREATE TABLE bounty_completed (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "user_id" uuid NOT NULL,
    "bounty_id" uuid NOT NULL,
    "completed_at" timestamp with time zone NOT NULL DEFAULT now()
);

CREATE TABLE last_gddl_update (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL,
    "updated_at" timestamp with time zone NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE level_custom_copies (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "level_id" uuid NOT NULL,
    "copy_id" integer NOT NULL,
    "added_by" uuid NOT NULL,
    "description" character varying,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "id_type" custom_id_type NOT NULL,
    "status" custom_id_status NOT NULL
);

CREATE TABLE level_notes (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "level_id" uuid NOT NULL,
    "note" text NOT NULL,
    "note_type" level_notes_type NOT NULL,
    "timestamp" timestamp with time zone,
    "added_by" uuid NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE level_updates (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "level_id" uuid NOT NULL,
    "changelog" text,
    "update_type" level_update_type NOT NULL,
    "timestamp" timestamp with time zone NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE levels (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT uuid_generate_v4(),
    "position" integer,
    "name" character varying NOT NULL,
    "publisher_id" uuid NOT NULL,
    "points" integer NOT NULL DEFAULT 0,
    "level_id" integer NOT NULL,
    "two_player" boolean NOT NULL,
    "tags" text[] NOT NULL DEFAULT '{}'::text[],
    "description" character varying,
    "song" integer,
    "edel_enjoyment" double precision,
    "is_edel_pending" boolean NOT NULL DEFAULT false,
    "gddl_tier" double precision,
    "nlw_tier" character varying,
    "status" level_status NOT NULL,
    "requires_raw_footage" boolean NOT NULL DEFAULT false,
    "nlw_tier_estimate" character varying
);

CREATE TABLE levels_created (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "level_id" uuid NOT NULL,
    "user_id" uuid NOT NULL
);

CREATE TABLE pack_levels (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "pack_id" uuid NOT NULL,
    "level_id" uuid NOT NULL
);

CREATE TABLE pack_tiers (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT uuid_generate_v4(),
    "name" character varying NOT NULL,
    "color" character varying NOT NULL,
    "placement" integer NOT NULL DEFAULT 0
);

CREATE TABLE packs (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT uuid_generate_v4(),
    "name" character varying NOT NULL,
    "tier" uuid NOT NULL
);

CREATE TABLE position_history (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "i" integer NOT NULL DEFAULT nextval('public.position_history_i_seq'::regclass),
    "new_position" integer,
    "old_position" integer,
    "affected_level" uuid NOT NULL,
    "level_above" uuid,
    "level_below" uuid,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "old_status" level_status,
    "new_status" level_status NOT NULL
);

CREATE TABLE position_history_full_view (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "ord" integer NOT NULL,
    "affected_level" uuid NOT NULL,
    "position" integer,
    "moved" boolean NOT NULL,
    "status" level_status NOT NULL,
    "action_at" timestamp with time zone NOT NULL,
    "cause" uuid NOT NULL,
    "pos_diff" integer
);

CREATE TABLE records (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT uuid_generate_v4(),
    "level_id" uuid NOT NULL,
    "submitted_by" uuid NOT NULL,
    "mobile" boolean NOT NULL DEFAULT false,
    "video_url" character varying NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "updated_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "is_verification" boolean NOT NULL DEFAULT false,
    "completion_time" bigint,
    "hide_video" boolean NOT NULL DEFAULT false,
    "submission_id" uuid NOT NULL,
    "achieved_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE submission_daily_level_stats (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "day" date NOT NULL,
    "level_id" uuid NOT NULL,
    "submitted" bigint NOT NULL DEFAULT 0,
    "accepted" bigint NOT NULL DEFAULT 0,
    "denied" bigint NOT NULL DEFAULT 0,
    "under_consideration" bigint NOT NULL DEFAULT 0,
    "reviewed" bigint NOT NULL DEFAULT 0
);

CREATE TABLE submission_daily_reviewer_stats (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "day" date NOT NULL,
    "reviewer_id" uuid NOT NULL,
    "accepted" bigint NOT NULL DEFAULT 0,
    "denied" bigint NOT NULL DEFAULT 0,
    "under_consideration" bigint NOT NULL DEFAULT 0,
    "reviewed" bigint NOT NULL DEFAULT 0
);

CREATE TABLE submission_daily_total_stats (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "day" date NOT NULL,
    "submitted" bigint NOT NULL DEFAULT 0,
    "accepted" bigint NOT NULL DEFAULT 0,
    "denied" bigint NOT NULL DEFAULT 0,
    "under_consideration" bigint NOT NULL DEFAULT 0,
    "reviewed" bigint NOT NULL DEFAULT 0
);

CREATE TABLE submission_history (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL,
    "submission_id" uuid NOT NULL,
    "reviewer_notes" text,
    "status" submission_status NOT NULL,
    "timestamp" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "user_notes" text,
    "reviewer_id" uuid,
    "mobile" boolean,
    "custom_copy_id" integer,
    "video_url" character varying,
    "raw_url" character varying,
    "mod_menu" character varying,
    "priority" boolean,
    "private_reviewer_notes" text,
    "locked" boolean,
    "completion_time" bigint
);

CREATE TABLE submissions (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT uuid_generate_v4(),
    "level_id" uuid NOT NULL,
    "submitted_by" uuid NOT NULL,
    "mobile" boolean NOT NULL DEFAULT false,
    "custom_copy_id" integer,
    "video_url" character varying NOT NULL,
    "raw_url" character varying,
    "reviewer_id" uuid,
    "priority" boolean NOT NULL DEFAULT false,
    "reviewer_notes" character varying,
    "user_notes" character varying,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "status" submission_status NOT NULL DEFAULT 'Pending'::submission_status,
    "mod_menu" character varying,
    "updated_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp(),
    "completion_time" bigint,
    "private_reviewer_notes" text,
    "locked" boolean NOT NULL DEFAULT false,
    "priority_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp()
);

CREATE TABLE submissions_enabled (
    list_id SMALLINT NOT NULL REFERENCES lists(id),
    "id" uuid NOT NULL DEFAULT gen_random_uuid(),
    "enabled" boolean NOT NULL,
    "moderator" uuid NOT NULL,
    "created_at" timestamp with time zone NOT NULL DEFAULT clock_timestamp()
);

-- copy data

INSERT INTO bounties (list_id, "id", "level_id", "bounty_type", "bounty_difficulty", "start_date", "end_date", "target_submissions", "is_target_public")
SELECT 1, "id", "level_id", "bounty_type", "bounty_difficulty", "start_date", "end_date", "target_submissions", "is_target_public" FROM aredl.bounties;
INSERT INTO bounties (list_id, "id", "level_id", "bounty_type", "bounty_difficulty", "start_date", "end_date", "target_submissions", "is_target_public")
SELECT 2, "id", "level_id", "bounty_type", "bounty_difficulty", "start_date", "end_date", "target_submissions", "is_target_public" FROM arepl.bounties;
INSERT INTO bounty_completed (list_id, "user_id", "bounty_id", "completed_at")
SELECT 1, "user_id", "bounty_id", "completed_at" FROM aredl.bounty_completed;
INSERT INTO bounty_completed (list_id, "user_id", "bounty_id", "completed_at")
SELECT 2, "user_id", "bounty_id", "completed_at" FROM arepl.bounty_completed;
INSERT INTO last_gddl_update (list_id, "id", "updated_at")
SELECT 1, "id", "updated_at" FROM aredl.last_gddl_update;
INSERT INTO last_gddl_update (list_id, "id", "updated_at")
SELECT 2, "id", "updated_at" FROM arepl.last_gddl_update;
INSERT INTO level_custom_copies (list_id, "id", "level_id", "copy_id", "added_by", "description", "created_at", "id_type", "status")
SELECT 1, "id", "level_id", "copy_id", "added_by", "description", "created_at", "id_type"::text::custom_id_type, "status"::text::custom_id_status FROM aredl.level_custom_copies;
INSERT INTO level_custom_copies (list_id, "id", "level_id", "copy_id", "added_by", "description", "created_at", "id_type", "status")
SELECT 2, "id", "level_id", "copy_id", "added_by", "description", "created_at", "id_type"::text::custom_id_type, "status"::text::custom_id_status FROM arepl.level_custom_copies;
INSERT INTO level_notes (list_id, "id", "level_id", "note", "note_type", "timestamp", "added_by", "created_at")
SELECT 1, "id", "level_id", "note", "note_type"::text::level_notes_type, "timestamp", "added_by", "created_at" FROM aredl.level_notes;
INSERT INTO level_notes (list_id, "id", "level_id", "note", "note_type", "timestamp", "added_by", "created_at")
SELECT 2, "id", "level_id", "note", "note_type"::text::level_notes_type, "timestamp", "added_by", "created_at" FROM arepl.level_notes;
INSERT INTO level_updates (list_id, "id", "level_id", "changelog", "update_type", "timestamp", "created_at")
SELECT 1, "id", "level_id", "changelog", "update_type"::text::level_update_type, "timestamp", "created_at" FROM aredl.level_updates;
INSERT INTO level_updates (list_id, "id", "level_id", "changelog", "update_type", "timestamp", "created_at")
SELECT 2, "id", "level_id", "changelog", "update_type"::text::level_update_type, "timestamp", "created_at" FROM arepl.level_updates;
INSERT INTO levels (list_id, "id", "position", "name", "publisher_id", "points", "level_id", "two_player", "tags", "description", "song", "edel_enjoyment", "is_edel_pending", "gddl_tier", "nlw_tier", "status", "requires_raw_footage", "nlw_tier_estimate")
SELECT 1, "id", "position", "name", "publisher_id", "points", "level_id", "two_player", "tags", "description", "song", "edel_enjoyment", "is_edel_pending", "gddl_tier", "nlw_tier", "status", "requires_raw_footage", "nlw_tier_estimate" FROM aredl.levels;
INSERT INTO levels (list_id, "id", "position", "name", "publisher_id", "points", "level_id", "two_player", "tags", "description", "song", "edel_enjoyment", "is_edel_pending", "gddl_tier", "nlw_tier", "status", "requires_raw_footage", "nlw_tier_estimate")
SELECT 2, "id", "position", "name", "publisher_id", "points", "level_id", "two_player", "tags", "description", "song", "edel_enjoyment", "is_edel_pending", "gddl_tier", "nlw_tier", "status", "requires_raw_footage", "nlw_tier_estimate" FROM arepl.levels;
INSERT INTO levels_created (list_id, "level_id", "user_id")
SELECT 1, "level_id", "user_id" FROM aredl.levels_created;
INSERT INTO levels_created (list_id, "level_id", "user_id")
SELECT 2, "level_id", "user_id" FROM arepl.levels_created;
INSERT INTO pack_levels (list_id, "pack_id", "level_id")
SELECT 1, "pack_id", "level_id" FROM aredl.pack_levels;
INSERT INTO pack_levels (list_id, "pack_id", "level_id")
SELECT 2, "pack_id", "level_id" FROM arepl.pack_levels;
INSERT INTO pack_tiers (list_id, "id", "name", "color", "placement")
SELECT 1, "id", "name", "color", "placement" FROM aredl.pack_tiers;
INSERT INTO pack_tiers (list_id, "id", "name", "color", "placement")
SELECT 2, "id", "name", "color", "placement" FROM arepl.pack_tiers;
INSERT INTO packs (list_id, "id", "name", "tier")
SELECT 1, "id", "name", "tier" FROM aredl.packs;
INSERT INTO packs (list_id, "id", "name", "tier")
SELECT 2, "id", "name", "tier" FROM arepl.packs;
INSERT INTO position_history (list_id, "i", "new_position", "old_position", "affected_level", "level_above", "level_below", "created_at", "old_status", "new_status")
SELECT 1, "i", "new_position", "old_position", "affected_level", "level_above", "level_below", "created_at", "old_status", "new_status" FROM aredl.position_history;
INSERT INTO position_history (list_id, "i", "new_position", "old_position", "affected_level", "level_above", "level_below", "created_at", "old_status", "new_status")
SELECT 2, "i", "new_position", "old_position", "affected_level", "level_above", "level_below", "created_at", "old_status", "new_status" FROM arepl.position_history;
INSERT INTO position_history_full_view (list_id, "ord", "affected_level", "position", "moved", "status", "action_at", "cause", "pos_diff")
SELECT 1, "ord", "affected_level", "position", "moved", "status", "action_at", "cause", "pos_diff" FROM aredl.position_history_full_view;
INSERT INTO position_history_full_view (list_id, "ord", "affected_level", "position", "moved", "status", "action_at", "cause", "pos_diff")
SELECT 2, "ord", "affected_level", "position", "moved", "status", "action_at", "cause", "pos_diff" FROM arepl.position_history_full_view;
INSERT INTO records (list_id, "id", "level_id", "submitted_by", "mobile", "video_url", "created_at", "updated_at", "is_verification", "completion_time", "hide_video", "submission_id", "achieved_at")
SELECT 1, "id", "level_id", "submitted_by", "mobile", "video_url", "created_at", "updated_at", "is_verification", NULL::bigint, "hide_video", "submission_id", "achieved_at" FROM aredl.records;
INSERT INTO records (list_id, "id", "level_id", "submitted_by", "mobile", "video_url", "created_at", "updated_at", "is_verification", "completion_time", "hide_video", "submission_id", "achieved_at")
SELECT 2, "id", "level_id", "submitted_by", "mobile", "video_url", "created_at", "updated_at", "is_verification", "completion_time", "hide_video", "submission_id", "achieved_at" FROM arepl.records;
INSERT INTO submission_daily_level_stats (list_id, "day", "level_id", "submitted", "accepted", "denied", "under_consideration", "reviewed")
SELECT 1, "day", "level_id", "submitted", "accepted", "denied", "under_consideration", "reviewed" FROM aredl.submission_daily_level_stats;
INSERT INTO submission_daily_level_stats (list_id, "day", "level_id", "submitted", "accepted", "denied", "under_consideration", "reviewed")
SELECT 2, "day", "level_id", "submitted", "accepted", "denied", "under_consideration", "reviewed" FROM arepl.submission_daily_level_stats;
INSERT INTO submission_daily_reviewer_stats (list_id, "day", "reviewer_id", "accepted", "denied", "under_consideration", "reviewed")
SELECT 1, "day", "reviewer_id", "accepted", "denied", "under_consideration", "reviewed" FROM aredl.submission_daily_reviewer_stats;
INSERT INTO submission_daily_reviewer_stats (list_id, "day", "reviewer_id", "accepted", "denied", "under_consideration", "reviewed")
SELECT 2, "day", "reviewer_id", "accepted", "denied", "under_consideration", "reviewed" FROM arepl.submission_daily_reviewer_stats;
INSERT INTO submission_daily_total_stats (list_id, "day", "submitted", "accepted", "denied", "under_consideration", "reviewed")
SELECT 1, "day", "submitted", "accepted", "denied", "under_consideration", "reviewed" FROM aredl.submission_daily_total_stats;
INSERT INTO submission_daily_total_stats (list_id, "day", "submitted", "accepted", "denied", "under_consideration", "reviewed")
SELECT 2, "day", "submitted", "accepted", "denied", "under_consideration", "reviewed" FROM arepl.submission_daily_total_stats;
INSERT INTO submission_history (list_id, "id", "submission_id", "reviewer_notes", "status", "timestamp", "user_notes", "reviewer_id", "mobile", "custom_copy_id", "video_url", "raw_url", "mod_menu", "priority", "private_reviewer_notes", "locked", "completion_time")
SELECT 1, "id", "submission_id", "reviewer_notes", "status", "timestamp", "user_notes", "reviewer_id", "mobile", "custom_copy_id", "video_url", "raw_url", "mod_menu", "priority", "private_reviewer_notes", "locked", NULL::bigint FROM aredl.submission_history;
INSERT INTO submission_history (list_id, "id", "submission_id", "reviewer_notes", "status", "timestamp", "user_notes", "reviewer_id", "mobile", "custom_copy_id", "video_url", "raw_url", "mod_menu", "priority", "private_reviewer_notes", "locked", "completion_time")
SELECT 2, "id", "submission_id", "reviewer_notes", "status", "timestamp", "user_notes", "reviewer_id", "mobile", "custom_copy_id", "video_url", "raw_url", "mod_menu", "priority", "private_reviewer_notes", "locked", "completion_time" FROM arepl.submission_history;
INSERT INTO submissions (list_id, "id", "level_id", "submitted_by", "mobile", "custom_copy_id", "video_url", "raw_url", "reviewer_id", "priority", "reviewer_notes", "user_notes", "created_at", "status", "mod_menu", "updated_at", "completion_time", "private_reviewer_notes", "locked", "priority_at")
SELECT 1, "id", "level_id", "submitted_by", "mobile", "custom_copy_id", "video_url", "raw_url", "reviewer_id", "priority", "reviewer_notes", "user_notes", "created_at", "status", "mod_menu", "updated_at", NULL::bigint, "private_reviewer_notes", "locked", "priority_at" FROM aredl.submissions;
INSERT INTO submissions (list_id, "id", "level_id", "submitted_by", "mobile", "custom_copy_id", "video_url", "raw_url", "reviewer_id", "priority", "reviewer_notes", "user_notes", "created_at", "status", "mod_menu", "updated_at", "completion_time", "private_reviewer_notes", "locked", "priority_at")
SELECT 2, "id", "level_id", "submitted_by", "mobile", "custom_copy_id", "video_url", "raw_url", "reviewer_id", "priority", "reviewer_notes", "user_notes", "created_at", "status", "mod_menu", "updated_at", "completion_time", "private_reviewer_notes", "locked", "priority_at" FROM arepl.submissions;
INSERT INTO submissions_enabled (list_id, "id", "enabled", "moderator", "created_at")
SELECT 1, "id", "enabled", "moderator", "created_at" FROM aredl.submissions_enabled;
INSERT INTO submissions_enabled (list_id, "id", "enabled", "moderator", "created_at")
SELECT 2, "id", "enabled", "moderator", "created_at" FROM arepl.submissions_enabled;

-- primary keys

ALTER TABLE bounties ADD PRIMARY KEY (id);
ALTER TABLE bounties ADD UNIQUE (list_id, id);
ALTER TABLE bounty_completed ADD PRIMARY KEY (list_id, user_id, bounty_id);
ALTER TABLE last_gddl_update ADD PRIMARY KEY (id);
ALTER TABLE last_gddl_update ADD UNIQUE (list_id, id);
ALTER TABLE level_custom_copies ADD CHECK ((copy_id > 0));
ALTER TABLE level_custom_copies ADD PRIMARY KEY (id);
ALTER TABLE level_custom_copies ADD UNIQUE (list_id, id);
ALTER TABLE level_notes ADD PRIMARY KEY (id);
ALTER TABLE level_notes ADD UNIQUE (list_id, id);
ALTER TABLE level_updates ADD PRIMARY KEY (id);
ALTER TABLE level_updates ADD UNIQUE (list_id, id);
ALTER TABLE levels ADD CHECK ((level_id > 0));
ALTER TABLE levels ADD CHECK ((((status = ANY (ARRAY['Pending'::level_status, 'Removed'::level_status])) AND ("position" IS NULL)) OR ((status = ANY (ARRAY['MainList'::level_status, 'Legacy'::level_status])) AND ("position" IS NOT NULL))));
ALTER TABLE levels ADD PRIMARY KEY (id);
ALTER TABLE levels ADD UNIQUE (list_id, id);
ALTER TABLE levels ADD UNIQUE (list_id, level_id, two_player);
ALTER TABLE levels_created ADD PRIMARY KEY (list_id, level_id, user_id);
ALTER TABLE pack_levels ADD PRIMARY KEY (list_id, pack_id, level_id);
ALTER TABLE pack_tiers ADD PRIMARY KEY (id);
ALTER TABLE pack_tiers ADD UNIQUE (list_id, id);
ALTER TABLE packs ADD PRIMARY KEY (id);
ALTER TABLE packs ADD UNIQUE (list_id, id);
ALTER TABLE position_history ADD PRIMARY KEY (list_id, i);
ALTER TABLE position_history_full_view ADD PRIMARY KEY (list_id, ord, affected_level);
ALTER TABLE records ADD PRIMARY KEY (id);
ALTER TABLE records ADD UNIQUE (list_id, id);
ALTER TABLE records ADD UNIQUE (list_id, level_id, submitted_by);
ALTER TABLE records ADD CHECK ((list_id = 1 AND completion_time IS NULL) OR (list_id = 2 AND completion_time IS NOT NULL));
ALTER TABLE submission_daily_level_stats ADD CHECK ((accepted >= 0));
ALTER TABLE submission_daily_level_stats ADD CHECK ((denied >= 0));
ALTER TABLE submission_daily_level_stats ADD CHECK ((reviewed >= 0));
ALTER TABLE submission_daily_level_stats ADD CHECK ((submitted >= 0));
ALTER TABLE submission_daily_level_stats ADD CHECK ((under_consideration >= 0));
ALTER TABLE submission_daily_level_stats ADD PRIMARY KEY (list_id, day, level_id);
ALTER TABLE submission_daily_reviewer_stats ADD CHECK ((accepted >= 0));
ALTER TABLE submission_daily_reviewer_stats ADD CHECK ((denied >= 0));
ALTER TABLE submission_daily_reviewer_stats ADD CHECK ((reviewed >= 0));
ALTER TABLE submission_daily_reviewer_stats ADD CHECK ((under_consideration >= 0));
ALTER TABLE submission_daily_reviewer_stats ADD PRIMARY KEY (list_id, day, reviewer_id);
ALTER TABLE submission_daily_total_stats ADD CHECK ((accepted >= 0));
ALTER TABLE submission_daily_total_stats ADD CHECK ((denied >= 0));
ALTER TABLE submission_daily_total_stats ADD CHECK ((reviewed >= 0));
ALTER TABLE submission_daily_total_stats ADD CHECK ((submitted >= 0));
ALTER TABLE submission_daily_total_stats ADD CHECK ((under_consideration >= 0));
ALTER TABLE submission_daily_total_stats ADD PRIMARY KEY (list_id, day);
ALTER TABLE submission_history ADD PRIMARY KEY (id);
ALTER TABLE submission_history ADD UNIQUE (list_id, id);
ALTER TABLE submissions ADD PRIMARY KEY (id);
ALTER TABLE submissions ADD UNIQUE (list_id, id);
ALTER TABLE submissions ADD UNIQUE (list_id, level_id, submitted_by);
ALTER TABLE submissions ADD CHECK ((list_id = 1 AND completion_time IS NULL) OR (list_id = 2 AND completion_time IS NOT NULL));
ALTER TABLE submissions_enabled ADD PRIMARY KEY (id);
ALTER TABLE submissions_enabled ADD UNIQUE (list_id, id);
ALTER TABLE bounties ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON DELETE CASCADE;
ALTER TABLE bounty_completed ADD FOREIGN KEY ("list_id", "bounty_id") REFERENCES bounties ("list_id", "id") ON DELETE CASCADE;
ALTER TABLE bounty_completed ADD FOREIGN KEY ("user_id") REFERENCES users ("id") ON DELETE CASCADE;
ALTER TABLE last_gddl_update ADD FOREIGN KEY ("list_id", "id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE level_custom_copies ADD FOREIGN KEY ("added_by") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE SET NULL;
ALTER TABLE level_custom_copies ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE level_notes ADD FOREIGN KEY ("added_by") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE SET NULL;
ALTER TABLE level_notes ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE level_updates ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE levels ADD FOREIGN KEY ("publisher_id") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE SET NULL;
ALTER TABLE levels_created ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE levels_created ADD FOREIGN KEY ("user_id") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE pack_levels ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE pack_levels ADD FOREIGN KEY ("list_id", "pack_id") REFERENCES packs ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE packs ADD FOREIGN KEY ("list_id", "tier") REFERENCES pack_tiers ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE position_history ADD FOREIGN KEY ("list_id", "affected_level") REFERENCES levels ("list_id", "id");
ALTER TABLE position_history ADD FOREIGN KEY ("list_id", "level_above") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE;
ALTER TABLE position_history ADD FOREIGN KEY ("list_id", "level_below") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE;
ALTER TABLE position_history_full_view ADD FOREIGN KEY ("list_id", "affected_level") REFERENCES levels ("list_id", "id");
ALTER TABLE position_history_full_view ADD FOREIGN KEY ("list_id", "cause") REFERENCES levels ("list_id", "id");
ALTER TABLE records ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE records ADD FOREIGN KEY ("list_id", "submission_id") REFERENCES submissions ("list_id", "id") ON DELETE CASCADE;
ALTER TABLE records ADD FOREIGN KEY ("submitted_by") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE submission_daily_level_stats ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE submission_daily_reviewer_stats ADD FOREIGN KEY ("reviewer_id") REFERENCES users ("id") ON UPDATE CASCADE;
ALTER TABLE submission_history ADD FOREIGN KEY ("list_id", "submission_id") REFERENCES submissions ("list_id", "id") ON DELETE CASCADE;
ALTER TABLE submission_history ADD FOREIGN KEY ("reviewer_id") REFERENCES users ("id") ON UPDATE CASCADE;
ALTER TABLE submission_history ADD CHECK (list_id=2 OR completion_time IS NULL);
ALTER TABLE submissions ADD FOREIGN KEY ("list_id", "level_id") REFERENCES levels ("list_id", "id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE submissions ADD FOREIGN KEY ("reviewer_id") REFERENCES users ("id") ON DELETE SET NULL;
ALTER TABLE submissions ADD FOREIGN KEY ("submitted_by") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE CASCADE;
ALTER TABLE submissions_enabled ADD FOREIGN KEY ("moderator") REFERENCES users ("id") ON UPDATE CASCADE ON DELETE SET NULL;

ALTER SEQUENCE position_history_i_seq OWNED BY position_history.i;
SELECT setval('public.position_history_i_seq', GREATEST(COALESCE(MAX(i), 0), 1), COALESCE(MAX(i), 0) >= 1) FROM position_history;

-- indexes

CREATE INDEX users_country_id_idx ON users (country, id);
CREATE INDEX clan_members_clan_user_idx ON clan_members (clan_id, user_id);
CREATE INDEX bounty_completed_bounty_id ON bounty_completed USING btree (list_id, bounty_id);
CREATE INDEX bounty_completed_user_id ON bounty_completed USING btree (list_id, user_id);
CREATE INDEX levels_position_idx ON levels USING btree (list_id, "position") WHERE ("position" IS NOT NULL);
CREATE INDEX levels_publisher_position_idx ON levels USING btree (list_id, publisher_id, "position");
CREATE INDEX levels_created_user_level_idx ON levels_created USING btree (list_id, user_id, level_id);
CREATE INDEX pack_levels_level_pack_idx ON pack_levels USING btree (list_id, level_id, pack_id);
CREATE INDEX position_history_full_view_affected_ord_idx ON position_history_full_view
    (list_id, affected_level, ord DESC) INCLUDE (position, status);
CREATE INDEX position_history_full_view_level_action_idx ON position_history_full_view USING btree (list_id, affected_level, action_at DESC, ord DESC) INCLUDE ("position", status);
CREATE INDEX position_history_full_view_level_first_placed_idx ON position_history_full_view USING btree (list_id, affected_level, action_at, ord) INCLUDE ("position", status) WHERE ("position" IS NOT NULL);
CREATE INDEX records_fastest_time_idx ON records USING btree (list_id, level_id, completion_time, achieved_at, id) INCLUDE (submitted_by) WHERE (is_verification = false);
CREATE INDEX records_first_victor_idx ON records USING btree (list_id, level_id, achieved_at, created_at, id) INCLUDE (submitted_by) WHERE (is_verification = false);
CREATE INDEX records_submitted_by_level_idx ON records USING btree (list_id, submitted_by, level_id);
CREATE INDEX records_submission_id_idx ON records (submission_id);
CREATE INDEX submission_daily_level_stats_level_day_idx ON submission_daily_level_stats USING btree (list_id, level_id, day DESC);
CREATE INDEX submission_daily_reviewer_stats_reviewer_day_idx ON submission_daily_reviewer_stats USING btree (list_id, reviewer_id, day DESC);
CREATE INDEX hist_rev_ts_idx ON submission_history USING btree (list_id, reviewer_id, "timestamp");
CREATE INDEX hist_sub_ts_id_idx ON submission_history USING btree (list_id, submission_id, "timestamp", id);
CREATE INDEX hist_ts_idx ON submission_history USING btree (list_id, "timestamp");
CREATE INDEX submissions_status_idx ON submissions USING btree (list_id, status);
CREATE INDEX records_level_achieved_idx ON records
    (list_id, level_id, achieved_at, id) INCLUDE (submitted_by);
CREATE INDEX records_level_completion_idx ON records (list_id, level_id, completion_time, id);
CREATE INDEX submissions_queue_created_idx ON submissions (list_id, status, priority, created_at);
CREATE INDEX submissions_queue_priority_idx ON submissions (list_id, status, priority, priority_at);
CREATE INDEX submissions_status_updated_idx ON submissions (list_id, status, updated_at);

-- views

ANALYZE bounties;
ANALYZE bounty_completed;
ANALYZE last_gddl_update;
ANALYZE level_custom_copies;
ANALYZE level_notes;
ANALYZE level_updates;
ANALYZE levels;
ANALYZE levels_created;
ANALYZE pack_levels;
ANALYZE pack_tiers;
ANALYZE packs;
ANALYZE position_history;
ANALYZE position_history_full_view;
ANALYZE records;
ANALYZE submission_daily_level_stats;
ANALYZE submission_daily_reviewer_stats;
ANALYZE submission_daily_total_stats;
ANALYZE submission_history;
ANALYZE submissions;
ANALYZE submissions_enabled;

CREATE VIEW packs_points AS
SELECT p.list_id, p.id, p.name, p.tier, ROUND(SUM(l.points)::numeric * 0.5)::integer AS points
FROM packs p
JOIN pack_levels pl ON (pl.list_id, pl.pack_id) = (p.list_id, p.id)
JOIN levels l ON (l.list_id, l.id) = (pl.list_id, pl.level_id)
GROUP BY p.list_id, p.id;

CREATE VIEW completed_packs AS
WITH pcl AS (
    SELECT list_id, pack_id, COUNT(*) AS lc FROM pack_levels GROUP BY list_id, pack_id
)
SELECT r.list_id, r.submitted_by AS user_id, pl.pack_id, MAX(r.achieved_at) AS completed_at
FROM records r
JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
JOIN pack_levels pl ON (pl.list_id, pl.level_id) = (r.list_id, r.level_id)
JOIN pcl ON (pcl.list_id, pcl.pack_id) = (pl.list_id, pl.pack_id)
GROUP BY r.list_id, r.submitted_by, pl.pack_id, pcl.lc
HAVING COUNT(*) = pcl.lc;

CREATE VIEW user_pack_points AS
SELECT cp.list_id, cp.user_id, SUM(p.points)::integer AS points
FROM completed_packs cp
JOIN packs_points p ON (p.list_id, p.id) = (cp.list_id, cp.pack_id)
GROUP BY cp.list_id, cp.user_id;

CREATE VIEW clan_member_points AS
WITH clan_records AS (
    SELECT r.list_id, cm.clan_id, r.submitted_by, l.points,
           COUNT(*) OVER (PARTITION BY r.list_id, cm.clan_id, r.level_id) AS completion_count
    FROM records r
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 1
    JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status <> 'Removed'
)
SELECT list_id, clan_id, submitted_by, COUNT(*) AS completed_levels,
       SUM(points::double precision / completion_count::double precision) AS contributed_points
FROM clan_records GROUP BY list_id, clan_id, submitted_by;

CREATE VIEW badge_level_statistics AS
SELECT r.list_id, r.submitted_by, l.id, l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS position,
       l.position AS current_position, l.level_id, l.two_player, l.publisher_id,
       l.edel_enjoyment, l.nlw_tier, l.tags, r.is_verification, r.achieved_at,
       COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
       COALESCE(fastest_record.submitted_by = r.submitted_by, false) AS is_fastest_time
FROM records r
JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.action_at <= r.achieved_at
    ORDER BY ph.action_at DESC, ph.ord DESC LIMIT 1
) completed_state ON true
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.position IS NOT NULL
      AND completed_state.position IS NULL
      AND (completed_state.status IS NULL OR
           (completed_state.status = 'Pending' AND ph.action_at >= r.achieved_at))
    ORDER BY ph.action_at, ph.ord LIMIT 1
) first_placed_state ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE fr.list_id = r.list_id AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.achieved_at, fr.created_at, fr.id LIMIT 1
) first_record ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE r.list_id = 2 AND fr.list_id = r.list_id
      AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.completion_time, fr.achieved_at, fr.id LIMIT 1
) fastest_record ON true
WHERE completed_state.status IN ('MainList', 'Pending')
   OR (completed_state.status IS NULL AND first_placed_state.position IS NOT NULL);

CREATE MATERIALIZED VIEW user_leaderboard AS
WITH user_points AS (
    SELECT li.id AS list_id, u.id AS user_id, u.country,
           (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::integer AS total_points,
           COALESCE(pp.points, 0)::integer AS pack_points
    FROM lists li CROSS JOIN users u
    LEFT JOIN records r ON r.list_id = li.id AND r.submitted_by = u.id
    LEFT JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id) AND l.status = 'MainList'
    LEFT JOIN user_pack_points pp ON (pp.list_id, pp.user_id) = (r.list_id, r.submitted_by)
    WHERE u.ban_level = 0 GROUP BY li.id, u.id, u.country, pp.points
), hardest_position AS (
    SELECT r.list_id, r.submitted_by AS user_id, MIN(l.position) AS position
    FROM records r JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status = 'MainList' GROUP BY r.list_id, r.submitted_by
), hardest AS (
    SELECT hp.list_id, hp.user_id, hp.position, l.id AS level_id
    FROM hardest_position hp JOIN levels l
      ON l.list_id = hp.list_id AND l.position = hp.position AND l.status = 'MainList'
), level_count AS (
    SELECT r.list_id, r.submitted_by AS id, COUNT(*) AS c
    FROM records r JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status IN ('MainList', 'Pending') GROUP BY r.list_id, r.submitted_by
)
SELECT up.list_id,
       RANK() OVER (PARTITION BY up.list_id ORDER BY up.total_points DESC)::integer AS rank,
       RANK() OVER (PARTITION BY up.list_id ORDER BY up.total_points - up.pack_points DESC)::integer AS raw_rank,
       RANK() OVER (PARTITION BY up.list_id ORDER BY COALESCE(lc.c, 0) DESC)::integer AS extremes_rank,
       RANK() OVER (PARTITION BY up.list_id ORDER BY h.position)::integer AS hardest_rank,
       RANK() OVER (PARTITION BY up.list_id, up.country ORDER BY up.total_points DESC)::integer AS country_rank,
       RANK() OVER (PARTITION BY up.list_id, up.country ORDER BY up.total_points - up.pack_points DESC)::integer AS country_raw_rank,
       RANK() OVER (PARTITION BY up.list_id, up.country ORDER BY COALESCE(lc.c, 0) DESC)::integer AS country_extremes_rank,
       RANK() OVER (PARTITION BY up.list_id, up.country ORDER BY h.position)::integer AS country_hardest_rank,
       up.user_id, up.country, up.total_points, up.pack_points, h.level_id AS hardest,
       COALESCE(lc.c, 0)::integer AS extremes, cm.clan_id
FROM user_points up
LEFT JOIN hardest h ON (h.list_id, h.user_id) = (up.list_id, up.user_id)
LEFT JOIN level_count lc ON (lc.list_id, lc.id) = (up.list_id, up.user_id)
LEFT JOIN clan_members cm ON cm.user_id = up.user_id;
CREATE UNIQUE INDEX user_leaderboard_user_idx ON user_leaderboard (list_id, user_id);
CREATE INDEX user_leaderboard_rank_idx ON user_leaderboard (list_id, rank, user_id);

CREATE MATERIALIZED VIEW record_totals AS
SELECT li.id AS list_id, NULL::uuid AS level_id,
       COUNT(r.id) FILTER (WHERE r.is_verification = false) AS records,
       COUNT(r.id) FILTER (WHERE r.is_verification = true) AS verifications
FROM lists li
LEFT JOIN (records r JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2)
  ON r.list_id = li.id
GROUP BY li.id
UNION ALL
SELECT r.list_id, r.level_id,
       COUNT(*) FILTER (WHERE r.is_verification = false), COUNT(*) FILTER (WHERE r.is_verification = true)
FROM records r JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
GROUP BY r.list_id, r.level_id;
CREATE UNIQUE INDEX record_totals_idx ON record_totals
    (list_id, level_id);

CREATE MATERIALIZED VIEW submission_totals AS
SELECT li.id AS list_id, NULL::uuid AS level_id, COUNT(s.id) AS submissions,
       100.00::double precision AS percent_of_queue
FROM lists li LEFT JOIN submissions s ON s.list_id = li.id AND s.status = 'Pending'
GROUP BY li.id
UNION ALL
SELECT list_id, level_id, COUNT(*),
       ROUND(COUNT(*) * 100.0 / SUM(COUNT(*)) OVER (PARTITION BY list_id), 2)::double precision
FROM submissions WHERE status = 'Pending' GROUP BY list_id, level_id;
CREATE UNIQUE INDEX submission_totals_idx ON submission_totals (list_id, submissions DESC, level_id);

CREATE VIEW min_placement_country_records AS
WITH ranked AS (
    SELECT r.*, u.country AS country,
           ROW_NUMBER() OVER record_window AS order_pos,
           COUNT(*) OVER record_window AS completion_count
    FROM records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 1

    JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status <> 'Removed' AND u.country IS NOT NULL
    WINDOW record_window AS (
        PARTITION BY r.list_id, r.level_id, u.country
        ORDER BY r.achieved_at
        ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
    )
)
SELECT * FROM ranked WHERE order_pos = 1;

CREATE MATERIALIZED VIEW country_created_levels AS
SELECT l.list_id, u.country AS country, l.id AS level_id, lc.user_id AS creator_id, l.position AS order_pos
FROM levels_created lc
JOIN levels l ON (l.list_id, l.id) = (lc.list_id, lc.level_id)
JOIN users u ON u.id = lc.user_id
WHERE l.status <> 'Removed' AND u.country IS NOT NULL
UNION
SELECT l.list_id, u.country, l.id, l.publisher_id, l.position
FROM levels l
JOIN users u ON u.id = l.publisher_id
LEFT JOIN levels_created lc ON (lc.list_id, lc.level_id) = (l.list_id, l.id)
WHERE lc.level_id IS NULL AND l.status <> 'Removed' AND u.country IS NOT NULL;
CREATE UNIQUE INDEX country_created_levels_idx ON country_created_levels (list_id, country, order_pos, level_id, creator_id);

CREATE MATERIALIZED VIEW country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT r.list_id, u.country AS country, r.level_id
    FROM records r JOIN users u ON u.id = r.submitted_by

    JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE u.ban_level <= 1 AND u.country IS NOT NULL AND u.country <> 0 AND l.status IN ('MainList', 'Pending')
), level_points AS (
    SELECT c.list_id, c.country, COALESCE(SUM(l.points), 0)::integer AS level_points
    FROM completed_levels c JOIN levels l ON (l.list_id, l.id) = (c.list_id, c.level_id)
    GROUP BY c.list_id, c.country
), hardest_position AS (
    SELECT c.list_id, c.country, MIN(l.position) AS position
    FROM completed_levels c JOIN levels l ON (l.list_id, l.id) = (c.list_id, c.level_id)
    GROUP BY c.list_id, c.country
), hardest AS (
    SELECT hp.list_id, hp.country, hp.position, l.id AS level_id
    FROM hardest_position hp JOIN levels l
      ON l.list_id = hp.list_id AND l.position = hp.position AND l.status = 'MainList'
), level_count AS (
    SELECT list_id, country, COUNT(*) AS c FROM completed_levels GROUP BY list_id, country
), user_count AS (
    SELECT country, COUNT(*) AS c FROM users WHERE ban_level <= 1 AND country IS NOT NULL AND country <> 0 GROUP BY country
)
SELECT lp.list_id,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY lp.level_points DESC)::integer AS rank,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY COALESCE(lc.c, 0) DESC)::integer AS extremes_rank,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY h.position)::integer AS hardest_rank,
       lp.country, lp.level_points, COALESCE(uc.c, 0)::integer AS members_count,
       h.level_id AS hardest, COALESCE(lc.c, 0)::integer AS extremes
FROM level_points lp
LEFT JOIN hardest h ON (h.list_id, h.country) = (lp.list_id, lp.country)
LEFT JOIN level_count lc ON (lc.list_id, lc.country) = (lp.list_id, lp.country)
LEFT JOIN user_count uc ON uc.country = lp.country;
CREATE UNIQUE INDEX country_leaderboard_rank_idx ON country_leaderboard (list_id, rank, country);

CREATE VIEW min_placement_clans_records AS
WITH ranked AS (
    SELECT r.*, cm.clan_id AS clan_id,
           ROW_NUMBER() OVER (PARTITION BY r.list_id, r.level_id, cm.clan_id ORDER BY r.achieved_at) AS order_pos,
           COUNT(*) OVER (PARTITION BY r.list_id, r.level_id, cm.clan_id) AS completion_count
    FROM records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 1
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status <> 'Removed'
)
SELECT * FROM ranked WHERE order_pos = 1;

CREATE MATERIALIZED VIEW clans_created_levels AS
SELECT l.list_id, cm.clan_id AS clan_id, l.id AS level_id, lc.user_id AS creator_id, l.position AS order_pos
FROM levels_created lc
JOIN levels l ON (l.list_id, l.id) = (lc.list_id, lc.level_id)
JOIN clan_members cm ON cm.user_id = lc.user_id
WHERE l.status <> 'Removed'
UNION
SELECT l.list_id, cm.clan_id, l.id, l.publisher_id, l.position
FROM levels l
JOIN clan_members cm ON cm.user_id = l.publisher_id
LEFT JOIN levels_created lc ON (lc.list_id, lc.level_id) = (l.list_id, l.id)
WHERE lc.level_id IS NULL AND l.status <> 'Removed' ;
CREATE UNIQUE INDEX clans_created_levels_idx ON clans_created_levels (list_id, clan_id, order_pos, level_id, creator_id);

CREATE MATERIALIZED VIEW clans_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT r.list_id, cm.clan_id AS clan_id, r.level_id
    FROM records r JOIN users u ON u.id = r.submitted_by
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE u.ban_level <= 1  AND l.status IN ('MainList', 'Pending')
), level_points AS (
    SELECT c.list_id, c.clan_id, COALESCE(SUM(l.points), 0)::integer AS level_points
    FROM completed_levels c JOIN levels l ON (l.list_id, l.id) = (c.list_id, c.level_id)
    GROUP BY c.list_id, c.clan_id
), hardest_position AS (
    SELECT c.list_id, c.clan_id, MIN(l.position) AS position
    FROM completed_levels c JOIN levels l ON (l.list_id, l.id) = (c.list_id, c.level_id)
    GROUP BY c.list_id, c.clan_id
), hardest AS (
    SELECT hp.list_id, hp.clan_id, hp.position, l.id AS level_id
    FROM hardest_position hp JOIN levels l
      ON l.list_id = hp.list_id AND l.position = hp.position AND l.status = 'MainList'
), level_count AS (
    SELECT list_id, clan_id, COUNT(*) AS c FROM completed_levels GROUP BY list_id, clan_id
), user_count AS (
    SELECT clan_id, COUNT(*) AS c FROM clan_members GROUP BY clan_id
)
SELECT lp.list_id,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY lp.level_points DESC)::integer AS rank,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY COALESCE(lc.c, 0) DESC)::integer AS extremes_rank,
       RANK() OVER (PARTITION BY lp.list_id ORDER BY h.position)::integer AS hardest_rank,
       lp.clan_id, lp.level_points, COALESCE(uc.c, 0)::integer AS members_count,
       h.level_id AS hardest, COALESCE(lc.c, 0)::integer AS extremes
FROM level_points lp
LEFT JOIN hardest h ON (h.list_id, h.clan_id) = (lp.list_id, lp.clan_id)
LEFT JOIN level_count lc ON (lc.list_id, lc.clan_id) = (lp.list_id, lp.clan_id)
LEFT JOIN user_count uc ON uc.clan_id = lp.clan_id;

-- view indexes

CREATE UNIQUE INDEX clans_leaderboard_rank_idx ON clans_leaderboard (list_id, rank, clan_id);
CREATE INDEX submissions_user_created_idx ON submissions (list_id, submitted_by, created_at DESC);
CREATE INDEX submissions_updated_idx ON submissions (list_id, updated_at DESC);
CREATE INDEX user_leaderboard_country_rank_page_idx ON user_leaderboard (list_id, country, country_rank, user_id);
CREATE INDEX user_leaderboard_clan_rank_page_idx ON user_leaderboard (list_id, clan_id, rank, user_id);
CREATE INDEX user_leaderboard_raw_rank_page_idx ON user_leaderboard (list_id, raw_rank, user_id);
CREATE INDEX user_leaderboard_country_raw_rank_page_idx ON user_leaderboard (list_id, country, country_raw_rank, user_id);
CREATE INDEX user_leaderboard_clan_raw_rank_page_idx ON user_leaderboard (list_id, clan_id, raw_rank, user_id);
CREATE INDEX user_leaderboard_extremes_rank_page_idx ON user_leaderboard (list_id, extremes_rank, user_id);
CREATE INDEX user_leaderboard_country_extremes_rank_page_idx ON user_leaderboard (list_id, country, country_extremes_rank, user_id);
CREATE INDEX user_leaderboard_clan_extremes_rank_page_idx ON user_leaderboard (list_id, clan_id, extremes_rank, user_id);
CREATE INDEX user_leaderboard_hardest_rank_page_idx ON user_leaderboard (list_id, hardest_rank, user_id);
CREATE INDEX user_leaderboard_country_hardest_rank_page_idx ON user_leaderboard (list_id, country, country_hardest_rank, user_id);
CREATE INDEX user_leaderboard_clan_hardest_rank_page_idx ON user_leaderboard (list_id, clan_id, hardest_rank, user_id);

ANALYZE submissions;
ANALYZE records;
ANALYZE position_history_full_view;
ANALYZE user_leaderboard;

-- triggers

CREATE FUNCTION append_position_history_full_view(target_list_id smallint, history_entry_i integer)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
DECLARE
    history_ord INTEGER;
    current_ord INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('public.position_history'), target_list_id::integer);

    SELECT COUNT(*)::INTEGER
    INTO history_ord
    FROM (SELECT * FROM public.position_history WHERE list_id = target_list_id) AS position_history
    WHERE i <= history_entry_i;

    IF history_ord = 0 THEN
        RAISE EXCEPTION 'Position history entry % does not exist', history_entry_i;
    END IF;

    SELECT COALESCE(MAX(ord), 0)
    INTO current_ord
    FROM (SELECT * FROM public.position_history_full_view WHERE list_id = target_list_id) AS position_history_full_view;

    IF history_ord <> current_ord + 1 THEN
        RAISE EXCEPTION 'Cannot append position history ord %, current ord is %', history_ord, current_ord;
    END IF;

    INSERT INTO public.position_history_full_view (list_id, 
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
    WITH r AS (
        SELECT
            history_ord AS ord,
            ph.old_position,
            ph.new_position,
            ph.old_status,
            ph.new_status,
            COALESCE(ph.old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
            ph.new_status IN ('MainList', 'Legacy') AS new_placed,
            ph.created_at,
            ph.affected_level
        FROM (SELECT * FROM public.position_history WHERE list_id = target_list_id) AS ph
        WHERE ph.i = history_entry_i
    ),
    previous_state AS (
        SELECT DISTINCT ON (phv.affected_level)
            phv.affected_level,
            phv.position,
            phv.status
        FROM (SELECT * FROM public.position_history_full_view WHERE list_id = target_list_id) AS phv
        ORDER BY phv.affected_level, phv.ord DESC
    ),
    changed_existing AS (
        SELECT
            r.ord,
            ps.affected_level,
            CASE
                WHEN r.affected_level = ps.affected_level THEN r.new_position
                WHEN ps.status NOT IN ('MainList', 'Legacy') THEN ps.position
                WHEN NOT r.old_placed AND r.new_placed THEN
                    CASE WHEN ps.position >= r.new_position THEN ps.position + 1 ELSE ps.position END
                WHEN r.old_placed AND NOT r.new_placed THEN
                    CASE WHEN ps.position > r.old_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position < r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.old_position AND r.new_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position > r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.new_position AND r.old_position THEN ps.position + 1 ELSE ps.position END
                ELSE ps.position
            END AS position,
            ps.position AS prev_pos,
            CASE WHEN r.affected_level = ps.affected_level THEN r.new_status ELSE ps.status END AS status,
            ps.status AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
        FROM previous_state ps
        CROSS JOIN r
    ),
    new_affected_level AS (
        SELECT
            r.ord,
            r.affected_level,
            r.new_position AS position,
            CAST(NULL AS INT) AS prev_pos,
            r.new_status AS status,
            CAST(NULL AS level_status) AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            false AS moved
        FROM r
        WHERE NOT EXISTS (
            SELECT 1
            FROM previous_state ps
            WHERE ps.affected_level = r.affected_level
        )
    ),
    filtered AS (
        SELECT *
        FROM changed_existing
        WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
        UNION ALL
        SELECT *
        FROM new_affected_level
    )
    SELECT
        target_list_id, f.ord,
        f.affected_level,
        f.position,
        f.moved,
        f.status,
        f.action_at,
        f.cause,
        f.position - prev.position AS pos_diff
    FROM filtered f
    LEFT JOIN LATERAL (
        SELECT phv.position
        FROM (SELECT * FROM public.position_history_full_view WHERE list_id = target_list_id) AS phv
        WHERE phv.affected_level = f.affected_level
        ORDER BY phv.ord DESC
        LIMIT 1
    ) prev ON true;
END;
$function$;

CREATE FUNCTION apply_submission_daily_stats_diff(target_list_id smallint, p_day date, p_reviewer_id uuid, p_level_id uuid, p_submitted bigint, p_accepted bigint, p_denied bigint, p_under_consideration bigint)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF p_submitted = 0
        AND p_accepted = 0
        AND p_denied = 0
        AND p_under_consideration = 0
    THEN
        RETURN;
    END IF;

    INSERT INTO public.submission_daily_total_stats (list_id, 
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (target_list_id, 
        p_day,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (list_id, day) DO UPDATE
    SET
        submitted = public.submission_daily_total_stats.submitted + EXCLUDED.submitted,
        accepted = public.submission_daily_total_stats.accepted + EXCLUDED.accepted,
        denied = public.submission_daily_total_stats.denied + EXCLUDED.denied,
        under_consideration = public.submission_daily_total_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = public.submission_daily_total_stats.reviewed + EXCLUDED.reviewed;

    INSERT INTO public.submission_daily_level_stats (list_id, 
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (target_list_id, 
        p_day,
        p_level_id,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (list_id, day, level_id) DO UPDATE
    SET
        submitted = public.submission_daily_level_stats.submitted + EXCLUDED.submitted,
        accepted = public.submission_daily_level_stats.accepted + EXCLUDED.accepted,
        denied = public.submission_daily_level_stats.denied + EXCLUDED.denied,
        under_consideration = public.submission_daily_level_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = public.submission_daily_level_stats.reviewed + EXCLUDED.reviewed;

    IF p_reviewer_id IS NOT NULL
        AND (
            p_accepted <> 0
            OR p_denied <> 0
            OR p_under_consideration <> 0
        )
    THEN
        INSERT INTO public.submission_daily_reviewer_stats (list_id, 
            day,
            reviewer_id,
            accepted,
            denied,
            under_consideration,
            reviewed
        )
        VALUES (target_list_id, 
            p_day,
            p_reviewer_id,
            p_accepted,
            p_denied,
            p_under_consideration,
            p_accepted + p_denied + p_under_consideration
        )
        ON CONFLICT (list_id, day, reviewer_id) DO UPDATE
        SET
            accepted = public.submission_daily_reviewer_stats.accepted + EXCLUDED.accepted,
            denied = public.submission_daily_reviewer_stats.denied + EXCLUDED.denied,
            under_consideration = public.submission_daily_reviewer_stats.under_consideration + EXCLUDED.under_consideration,
            reviewed = public.submission_daily_reviewer_stats.reviewed + EXCLUDED.reviewed;
    END IF;

END;
$function$;

CREATE FUNCTION level_move()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('public.position_history'), NEW.list_id::integer);

    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE public.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE list_id = NEW.list_id AND id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND (
          (NOT old_placed AND new_placed AND position >= NEW.position)
          OR
          (old_placed AND NOT new_placed AND position > OLD.position)
          OR
          (old_placed AND new_placed AND position BETWEEN LEAST(NEW.position, OLD.position) AND GREATEST(NEW.position, OLD.position))
      );

    IF new_placed THEN
        above := (
            SELECT id
            FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels
            WHERE list_id = NEW.list_id AND id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels
            WHERE list_id = NEW.list_id AND id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO public.position_history(list_id, new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.list_id, NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM public.append_position_history_full_view(NEW.list_id, history_i);

    RETURN NULL;
END;
$function$;

CREATE FUNCTION level_place()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        RETURN NULL;
    END IF;

    UPDATE public.levels
    SET position = position + 1
    WHERE list_id = NEW.list_id AND id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND position >= NEW.position;

    RETURN NULL;
END;
$function$;

CREATE FUNCTION level_place_history()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('public.position_history'), NEW.list_id::integer);

    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO public.position_history(list_id, new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.list_id, NEW.position, NULL, NULL, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM public.append_position_history_full_view(NEW.list_id, history_i);

    RETURN NULL;
END;
$function$;

CREATE FUNCTION levels_points_after_insert() RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE affected_list smallint;
BEGIN
    FOR affected_list IN SELECT DISTINCT list_id FROM inserted_levels ORDER BY list_id LOOP
        PERFORM public.recalculate_points(affected_list);
    END LOOP;
    RETURN NULL;
END $$;

CREATE FUNCTION levels_points_before_insert()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN public.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels WHERE status = 'MainList') + 1 AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$function$;

CREATE FUNCTION levels_points_before_update()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN public.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM (SELECT * FROM public.levels WHERE list_id = NEW.list_id) AS levels WHERE status = 'MainList') AS INT) + CASE WHEN OLD.status = 'MainList' THEN 0 ELSE 1 END)
        ELSE 0
    END;
    RETURN NEW;
END;
$function$;

CREATE FUNCTION max_list_pos(target_list_id smallint)
 RETURNS integer
 LANGUAGE sql
AS $function$
    SELECT COALESCE(MAX(position), 0) FROM (SELECT * FROM public.levels WHERE list_id = target_list_id) AS levels WHERE status = 'MainList';
$function$;

CREATE FUNCTION max_list_pos_legacy(target_list_id smallint)
 RETURNS integer
 LANGUAGE sql
AS $function$
    SELECT COALESCE(MAX(position), 0) FROM (SELECT * FROM public.levels WHERE list_id = target_list_id) AS levels WHERE status IN ('MainList', 'Legacy');
$function$;

CREATE FUNCTION point_formula(pos integer, level_count integer)
 RETURNS integer
 LANGUAGE plpgsql
AS $function$
DECLARE
    a float;
    b float;
BEGIN
    IF pos > level_count THEN
        return 0;
    END IF;
    IF level_count <= 1 THEN
        return 500;
    END IF;
    b := (level_count - 1) * 0.0005832492374192;
    a := 6000 * sqrt(b);
    return ROUND((a / sqrt((CAST(pos AS float) - 1) / 50 + b) - 1000));
END
$function$;

CREATE FUNCTION rebuild_position_history_full_view(target_list_id smallint)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('public.position_history'), target_list_id::integer);

    DELETE FROM public.position_history_full_view WHERE list_id = target_list_id;

    INSERT INTO public.position_history_full_view (list_id, 
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
    WITH RECURSIVE ranked_history AS (
        SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, old_position, new_position, old_status, new_status,
               COALESCE(old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
               new_status IN ('MainList', 'Legacy') AS new_placed, created_at, affected_level
        FROM (SELECT * FROM public.position_history WHERE list_id = target_list_id) AS position_history
    ),
    full_history AS (
        SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) AS prev_pos,
               new_status AS status, CAST(NULL AS level_status) AS prev_status,
               created_at AS action_at, affected_level AS cause, false AS moved
        FROM ranked_history
        WHERE old_status IS NULL
        UNION
        SELECT
            r.i,
            h.id,
            CASE
                WHEN r.affected_level = h.id THEN r.new_position
                WHEN h.status NOT IN ('MainList', 'Legacy') THEN h.position
                WHEN NOT r.old_placed AND r.new_placed THEN
                    CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
                WHEN r.old_placed AND NOT r.new_placed THEN
                    CASE WHEN h.position > r.old_position THEN h.position - 1 ELSE h.position END
                WHEN r.old_position < r.new_position THEN
                    CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
                WHEN r.old_position > r.new_position THEN
                    CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
                ELSE h.position
            END AS position,
            h.position AS prev_pos,
            CASE WHEN r.affected_level = h.id THEN r.new_status ELSE h.status END AS status,
            h.status AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
        FROM ranked_history r
        INNER JOIN full_history h ON r.i = h.i + 1
    ),
    filtered AS (
        SELECT i::INTEGER AS ord, id AS affected_level, position, moved, status, action_at, cause
        FROM full_history
        WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
    )
    SELECT target_list_id, *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff
    FROM filtered;
END;
$function$;

CREATE FUNCTION rebuild_submission_daily_stats(target_list_id smallint)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
BEGIN
    DELETE FROM public.submission_daily_total_stats WHERE list_id = target_list_id;
DELETE FROM public.submission_daily_reviewer_stats WHERE list_id = target_list_id;
DELETE FROM public.submission_daily_level_stats WHERE list_id = target_list_id;

    INSERT INTO public.submission_daily_total_stats (list_id, 
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            h.reviewer_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM (SELECT * FROM public.submission_history WHERE list_id = target_list_id) AS h
        INNER JOIN (SELECT * FROM public.submissions WHERE list_id = target_list_id) AS s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status AND (target_list_id = 1 OR reviewer_id IS NOT NULL))::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status AND (target_list_id = 1 OR reviewer_id IS NOT NULL))::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day
    )
    SELECT target_list_id, day, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;

    INSERT INTO public.submission_daily_reviewer_stats (list_id, 
        day,
        reviewer_id,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    SELECT
        target_list_id, DATE(h.timestamp) AS day,
        h.reviewer_id,
        SUM((h.status = 'Accepted'::submission_status)::int)::bigint AS accepted,
        SUM((h.status = 'Denied'::submission_status)::int)::bigint AS denied,
        SUM((h.status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
        SUM(
            (h.status = 'Accepted'::submission_status)::int
            + (h.status = 'Denied'::submission_status)::int
            + (h.status = 'UnderConsideration'::submission_status)::int
        )::bigint AS reviewed
    FROM (SELECT * FROM public.submission_history WHERE list_id = target_list_id) AS h
    INNER JOIN (SELECT * FROM public.submissions WHERE list_id = target_list_id) AS s ON s.id = h.submission_id
    WHERE h.reviewer_id IS NOT NULL
    GROUP BY DATE(h.timestamp), h.reviewer_id
    HAVING SUM((h.status = 'Accepted'::submission_status)::int) <> 0
        OR SUM((h.status = 'Denied'::submission_status)::int) <> 0
        OR SUM((h.status = 'UnderConsideration'::submission_status)::int) <> 0;

    INSERT INTO public.submission_daily_level_stats (list_id, 
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            h.reviewer_id, s.level_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM (SELECT * FROM public.submission_history WHERE list_id = target_list_id) AS h
        INNER JOIN (SELECT * FROM public.submissions WHERE list_id = target_list_id) AS s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            level_id,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status AND (target_list_id = 1 OR reviewer_id IS NOT NULL))::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status AND (target_list_id = 1 OR reviewer_id IS NOT NULL))::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day, level_id
    )
    SELECT target_list_id, day, level_id, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;
END;
$function$;

CREATE FUNCTION recalculate_points(target_list_id smallint)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
BEGIN
    UPDATE public.levels
    SET points = CASE
        WHEN status = 'MainList' THEN public.point_formula(position, CAST((SELECT COUNT(*) FROM (SELECT * FROM public.levels WHERE list_id = target_list_id) AS levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END WHERE list_id = target_list_id;
END;
$function$;

CREATE FUNCTION submission_is_only_claim_toggle(old_submission submissions, new_submission submissions)
 RETURNS boolean
 LANGUAGE plpgsql
AS $function$
BEGIN
    RETURN new_submission.status <> old_submission.status
        AND (
            (old_submission.status = 'Pending' AND new_submission.status = 'Claimed')
            OR (old_submission.status = 'Claimed' AND new_submission.status = 'Pending')
        )
        AND (to_jsonb(new_submission) - 'updated_at' - 'status' - 'reviewer_id')
            IS NOT DISTINCT FROM
            (to_jsonb(old_submission) - 'updated_at' - 'status' - 'reviewer_id');
END;
$function$;

CREATE FUNCTION submission_log_history()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO public.submission_history (list_id, id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (NEW.list_id, uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF public.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;

        INSERT INTO public.submission_history (list_id, id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (NEW.list_id, uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$function$;

CREATE FUNCTION submission_sync_record()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
BEGIN
    IF NEW.status = 'Accepted' THEN
        INSERT INTO public.records AS r (list_id, 
            level_id,
            submitted_by,
            mobile,
            video_url,
			completion_time,
            submission_id
        )
        VALUES (NEW.list_id, 
            NEW.level_id,
            NEW.submitted_by,
            NEW.mobile,
            NEW.video_url,
			NEW.completion_time,
            NEW.id
        )
        ON CONFLICT (list_id, level_id, submitted_by)
        DO UPDATE SET
            mobile = EXCLUDED.mobile,
            video_url = EXCLUDED.video_url,
			completion_time = EXCLUDED.completion_time;
			
    END IF;

    RETURN NEW;
END;
$function$;

CREATE FUNCTION submission_updated_at()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    update_timestamp timestamptz;
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        update_timestamp = CLOCK_TIMESTAMP();

        IF OLD.priority = FALSE AND NEW.priority = TRUE THEN
            NEW.priority_at = update_timestamp;
        END IF;

        NEW.updated_at = update_timestamp;
    END IF;

    RETURN NEW;
END;
$function$;

CREATE FUNCTION update_submission_daily_stats_from_submission()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    stats_day date;
    submitted_diff bigint;
    accepted_diff bigint;
    denied_diff bigint;
    under_consideration_diff bigint;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF public.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;
    END IF;

    stats_day = DATE(CLOCK_TIMESTAMP());
    submitted_diff = CASE
        WHEN NEW.status = 'Pending'::submission_status
             AND (TG_OP = 'INSERT' OR OLD.status IS DISTINCT FROM 'Pending'::submission_status)
        THEN 1
        ELSE 0
    END;
    accepted_diff = (NEW.status = 'Accepted'::submission_status AND (NEW.list_id = 1 OR NEW.reviewer_id IS NOT NULL))::int;
    denied_diff = (NEW.status = 'Denied'::submission_status)::int;
    under_consideration_diff = (NEW.status = 'UnderConsideration'::submission_status)::int;

    PERFORM public.apply_submission_daily_stats_diff(NEW.list_id, 
        stats_day,
        NEW.reviewer_id,
        NEW.level_id,
        submitted_diff,
        accepted_diff,
        denied_diff,
        under_consideration_diff
    );

    RETURN NEW;
END;
$function$;

CREATE FUNCTION validate_position_insert()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status
            USING ERRCODE = 'check_violation';
    END IF;

    IF NEW.status = 'MainList' THEN
        highestPos := public.max_list_pos(NEW.list_id) + 1;
        lowestPos := 1;
    ELSE
        highestPos := public.max_list_pos_legacy(NEW.list_id) + 1;
        lowestPos := public.max_list_pos(NEW.list_id) + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$function$;

CREATE FUNCTION validate_position_update()
 RETURNS trigger
 LANGUAGE plpgsql
AS $function$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF OLD.status NOT IN ('MainList', 'Legacy') THEN
        NEW.requires_raw_footage := FALSE;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status
            USING ERRCODE = 'check_violation';
    END IF;

    IF NEW.status = 'MainList' THEN
        IF OLD.status = 'MainList' THEN
            highestPos := public.max_list_pos(NEW.list_id);
        ELSE
            highestPos := public.max_list_pos(NEW.list_id) + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := public.max_list_pos(NEW.list_id);
        ELSE
            lowestPos := public.max_list_pos(NEW.list_id) + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := public.max_list_pos_legacy(NEW.list_id);
        ELSE
            highestPos := public.max_list_pos_legacy(NEW.list_id) + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$function$;

CREATE TRIGGER level_move AFTER UPDATE OF "position", status ON levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION level_move();
CREATE TRIGGER level_place AFTER INSERT ON levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION level_place();
CREATE TRIGGER level_place_history AFTER INSERT ON levels FOR EACH ROW EXECUTE FUNCTION level_place_history();
CREATE TRIGGER levels_points_after_insert AFTER INSERT ON levels REFERENCING NEW TABLE AS inserted_levels FOR EACH STATEMENT EXECUTE FUNCTION levels_points_after_insert();
CREATE TRIGGER levels_points_before_insert BEFORE INSERT ON levels FOR EACH ROW EXECUTE FUNCTION levels_points_before_insert();
CREATE TRIGGER levels_points_before_update BEFORE UPDATE OF "position", status ON levels FOR EACH ROW EXECUTE FUNCTION levels_points_before_update();
CREATE TRIGGER validate_position_insert BEFORE INSERT ON levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION validate_position_insert();
CREATE TRIGGER validate_position_update BEFORE UPDATE OF "position", status ON levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION validate_position_update();
CREATE TRIGGER submission_daily_stats_ins AFTER INSERT ON submissions FOR EACH ROW EXECUTE FUNCTION update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_daily_stats_upd AFTER UPDATE ON submissions FOR EACH ROW EXECUTE FUNCTION update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_log_history_ins AFTER INSERT ON submissions FOR EACH ROW EXECUTE FUNCTION submission_log_history();
CREATE TRIGGER submission_log_history_upd AFTER UPDATE ON submissions FOR EACH ROW EXECUTE FUNCTION submission_log_history();
CREATE TRIGGER submission_sync_record_ins AFTER INSERT ON submissions FOR EACH ROW EXECUTE FUNCTION submission_sync_record();
CREATE TRIGGER submission_sync_record_upd AFTER UPDATE OF status, mobile, video_url, completion_time ON submissions FOR EACH ROW EXECUTE FUNCTION submission_sync_record();
CREATE TRIGGER submission_updated_at BEFORE UPDATE ON submissions FOR EACH ROW EXECUTE FUNCTION submission_updated_at();

-- block changing list id

CREATE FUNCTION keep_list_id() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.list_id IS DISTINCT FROM OLD.list_id THEN
        RAISE EXCEPTION 'list_id is immutable' USING ERRCODE='check_violation';
    END IF;
    RETURN NEW;
END $$;
CREATE TRIGGER a_keep_list_id BEFORE UPDATE OF list_id ON levels FOR EACH ROW EXECUTE FUNCTION keep_list_id();
CREATE TRIGGER a_keep_list_id BEFORE UPDATE OF list_id ON pack_tiers FOR EACH ROW EXECUTE FUNCTION keep_list_id();
CREATE TRIGGER a_keep_list_id BEFORE UPDATE OF list_id ON submission_daily_reviewer_stats FOR EACH ROW EXECUTE FUNCTION keep_list_id();
CREATE TRIGGER a_keep_list_id BEFORE UPDATE OF list_id ON submission_daily_total_stats FOR EACH ROW EXECUTE FUNCTION keep_list_id();
CREATE TRIGGER a_keep_list_id BEFORE UPDATE OF list_id ON submissions_enabled FOR EACH ROW EXECUTE FUNCTION keep_list_id();

CREATE FUNCTION lock_level_list() RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('public.position_history'), NEW.list_id::integer);
    RETURN NEW;
END $$;
CREATE TRIGGER a_lock_level_list BEFORE INSERT OR UPDATE OF position,status ON levels
    FOR EACH ROW EXECUTE FUNCTION lock_level_list();

CREATE OR REPLACE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
DECLARE previous_replication_role text := current_setting('session_replication_role');
BEGIN
  IF p_primary_user = p_secondary_user THEN
    RAISE EXCEPTION 'Cannot merge a user with themselves'
      USING ERRCODE = 'check_violation';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_primary_user) THEN
    RAISE EXCEPTION 'Primary user % does not exist', p_primary_user
      USING ERRCODE = 'foreign_key_violation';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_secondary_user) THEN
    RAISE EXCEPTION 'Secondary user % does not exist', p_secondary_user
      USING ERRCODE = 'foreign_key_violation';
  END IF;

  PERFORM set_config('session_replication_role', 'replica', true);

  ----- deduplicate conflicting submissions (a submission for the same level exists for both users) -----
    WITH pairs AS (
        -- if secondary has an accepted one but not primary, keep secondary. in other cases keep primary.
        SELECT
            primary_s.level_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN secondary_s.id
                ELSE primary_s.id
            END AS keep_submission_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN primary_s.id
                ELSE secondary_s.id
            END AS discard_submission_id
        FROM public.submissions primary_s
        JOIN public.submissions secondary_s
            ON secondary_s.list_id = primary_s.list_id AND secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE public.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM public.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM public.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- recalculate stats

    INSERT INTO public.submission_daily_reviewer_stats AS stats
        (list_id, day, reviewer_id, accepted, denied, under_consideration, reviewed)
    SELECT list_id, day, p_primary_user, accepted, denied, under_consideration, reviewed
    FROM public.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user
    ON CONFLICT (list_id, day, reviewer_id) DO UPDATE
    SET accepted = stats.accepted + EXCLUDED.accepted,
        denied = stats.denied + EXCLUDED.denied,
        under_consideration = stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = stats.reviewed + EXCLUDED.reviewed;

    DELETE FROM public.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user;

  ----- other deduplication

    DELETE FROM public.levels_created ac1
    USING public.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.list_id = ac2.list_id AND ac1.level_id = ac2.level_id
        AND ac2.user_id = p_primary_user;

    DELETE FROM clan_members cm1
    USING clan_members cm2
    WHERE cm1.user_id = p_secondary_user
        AND cm2.user_id = p_primary_user;

    DELETE FROM user_roles ur1
    USING user_roles ur2
    WHERE ur1.user_id = p_secondary_user
        AND ur1.role_id = ur2.role_id
        AND ur2.user_id = p_primary_user;

    DELETE FROM user_badges ub1
    USING user_badges ub2
    WHERE ub1.user_id = p_secondary_user
        AND ub1.badge_code = ub2.badge_code
        AND ub2.user_id = p_primary_user;

	DELETE FROM public.bounty_completed bc1
	USING public.bounty_completed bc2
	WHERE bc1.user_id = p_secondary_user
		AND bc1.list_id = bc2.list_id AND bc1.bounty_id = bc2.bounty_id
		AND bc2.user_id = p_primary_user;

    DELETE FROM oauth_connected_accounts ac1
    USING oauth_connected_accounts ac2
    WHERE ac1.user_id = p_secondary_user
      AND ac2.user_id = p_primary_user
      AND ac1.provider = ac2.provider;

  ----- change ownership

    UPDATE public.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE public.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE public.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE public.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE public.bounty_completed SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE public.submissions SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE public.submission_history SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE public.submissions_enabled SET moderator = p_primary_user WHERE moderator = p_secondary_user;
    UPDATE public.level_custom_copies SET added_by = p_primary_user WHERE added_by = p_secondary_user;
    UPDATE public.level_notes SET added_by = p_primary_user WHERE added_by = p_secondary_user;

    UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_badges SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE shifts SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE recurrent_shifts SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE oauth_connected_accounts
    SET user_id = p_primary_user
    WHERE user_id = p_secondary_user;

    UPDATE users
    SET featured_badge_code = secondary_user.featured_badge_code
    FROM users AS secondary_user
    WHERE users.id = p_primary_user
      AND secondary_user.id = p_secondary_user
      AND users.featured_badge_code IS NULL
      AND secondary_user.featured_badge_code IS NOT NULL;

    PERFORM set_config('session_replication_role', previous_replication_role, true);

  ----- log and delete

    INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
    SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
    FROM users WHERE id = p_secondary_user;

    UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

    DELETE FROM users WHERE id = p_secondary_user;
END;
$function$;

DROP SCHEMA aredl CASCADE;
DROP SCHEMA arepl CASCADE;
