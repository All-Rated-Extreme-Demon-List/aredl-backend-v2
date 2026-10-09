SET LOCAL work_mem = '64MB';

CREATE SCHEMA aredl;

CREATE SCHEMA arepl;

CREATE TYPE aredl.custom_id_status AS ENUM ( 'Published', 'Allowed', 'Banned' );

CREATE TYPE aredl.custom_id_type AS ENUM ( 'Bugfix', 'GlobedCopy', 'Ldm', 'Other' );

CREATE TYPE aredl.level_notes_type AS ENUM ( 'ReviewerNotes', 'PublicNotes', 'Other' );

CREATE TYPE aredl.level_update_type AS ENUM ( 'Buff', 'Nerf', 'Balance', 'BugFix', 'Other' );

CREATE TYPE arepl.custom_id_status AS ENUM ( 'Published', 'Allowed', 'Banned' );

CREATE TYPE arepl.custom_id_type AS ENUM ( 'Bugfix', 'GlobedCopy', 'Ldm', 'Other' );

CREATE TYPE arepl.level_notes_type AS ENUM ( 'ReviewerNotes', 'PublicNotes', 'Other' );

CREATE TYPE arepl.level_update_type AS ENUM ( 'Buff', 'Nerf', 'Balance', 'BugFix', 'Other' );

CREATE TABLE aredl.submissions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    level_id uuid NOT NULL,
    submitted_by uuid NOT NULL,
    mobile boolean DEFAULT false NOT NULL,
    custom_copy_id integer,
    video_url character varying NOT NULL,
    raw_url character varying,
    reviewer_id uuid,
    priority boolean DEFAULT false NOT NULL,
    reviewer_notes character varying,
    user_notes character varying,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    status public.submission_status DEFAULT 'Pending'::public.submission_status NOT NULL,
    mod_menu character varying,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    private_reviewer_notes text,
    locked boolean DEFAULT false NOT NULL,
    priority_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE arepl.submissions (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    level_id uuid NOT NULL,
    submitted_by uuid NOT NULL,
    mobile boolean DEFAULT false NOT NULL,
    custom_copy_id integer,
    video_url character varying NOT NULL,
    raw_url character varying,
    reviewer_id uuid,
    priority boolean DEFAULT false NOT NULL,
    reviewer_notes character varying,
    user_notes character varying,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    status public.submission_status DEFAULT 'Pending'::public.submission_status NOT NULL,
    mod_menu character varying,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    completion_time bigint DEFAULT 0 NOT NULL,
    private_reviewer_notes text,
    locked boolean DEFAULT false NOT NULL,
    priority_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE aredl.levels (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    position integer,
    name character varying NOT NULL,
    publisher_id uuid NOT NULL,
    points integer DEFAULT 0 NOT NULL,
    level_id integer NOT NULL,
    two_player boolean NOT NULL,
    tags text[] DEFAULT '{}'::text[] NOT NULL,
    description character varying,
    song integer,
    edel_enjoyment double precision,
    is_edel_pending boolean DEFAULT false NOT NULL,
    gddl_tier double precision,
    nlw_tier character varying,
    status public.level_status NOT NULL,
    requires_raw_footage boolean DEFAULT false NOT NULL,
    nlw_tier_estimate character varying,
    CONSTRAINT aredl_levels_level_id_check CHECK ((level_id > 0)),
    CONSTRAINT aredl_levels_status_position_check CHECK ((((status = ANY (ARRAY['Pending'::public.level_status, 'Removed'::public.level_status])) AND (position IS NULL)) OR ((status = ANY (ARRAY['MainList'::public.level_status, 'Legacy'::public.level_status])) AND (position IS NOT NULL))))
);

CREATE TABLE aredl.position_history_full_view (
    ord integer NOT NULL,
    affected_level uuid NOT NULL,
    position integer,
    moved boolean NOT NULL,
    status public.level_status NOT NULL,
    action_at timestamp with time zone NOT NULL,
    cause uuid NOT NULL,
    pos_diff integer
);

CREATE TABLE aredl.records (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    level_id uuid NOT NULL,
    submitted_by uuid NOT NULL,
    mobile boolean DEFAULT false NOT NULL,
    video_url character varying NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    is_verification boolean DEFAULT false NOT NULL,
    hide_video boolean DEFAULT false NOT NULL,
    submission_id uuid NOT NULL,
    achieved_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE aredl.bounties (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    bounty_type public.bounty_type NOT NULL,
    bounty_difficulty public.bounty_difficulty NOT NULL,
    start_date timestamp with time zone NOT NULL,
    end_date timestamp with time zone,
    target_submissions integer,
    is_target_public boolean DEFAULT false NOT NULL
);

CREATE TABLE aredl.bounty_completed (
    user_id uuid NOT NULL,
    bounty_id uuid NOT NULL,
    completed_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE aredl.levels_created (
    level_id uuid NOT NULL,
    user_id uuid NOT NULL
);

CREATE TABLE aredl.pack_levels (
    pack_id uuid NOT NULL,
    level_id uuid NOT NULL
);

CREATE TABLE aredl.last_gddl_update (
    id uuid NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE aredl.level_custom_copies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    copy_id integer NOT NULL,
    added_by uuid NOT NULL,
    description character varying,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    id_type aredl.custom_id_type NOT NULL,
    status aredl.custom_id_status NOT NULL,
    CONSTRAINT level_ldms_ldm_id_check CHECK ((copy_id > 0))
);

CREATE TABLE aredl.level_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    note text NOT NULL,
    note_type aredl.level_notes_type NOT NULL,
    timestamp timestamp with time zone,
    added_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE aredl.level_updates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    changelog text,
    update_type aredl.level_update_type NOT NULL,
    timestamp timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE aredl.pack_tiers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying NOT NULL,
    color character varying NOT NULL,
    placement integer DEFAULT 0 NOT NULL
);

CREATE TABLE aredl.packs (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying NOT NULL,
    tier uuid NOT NULL
);

CREATE TABLE aredl.position_history (
    i integer NOT NULL,
    new_position integer,
    old_position integer,
    affected_level uuid NOT NULL,
    level_above uuid,
    level_below uuid,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    old_status public.level_status,
    new_status public.level_status NOT NULL
);

CREATE SEQUENCE aredl.position_history_i_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE aredl.position_history_i_seq OWNED BY aredl.position_history.i;

CREATE TABLE aredl.submission_daily_level_stats (
    day date NOT NULL,
    level_id uuid NOT NULL,
    submitted bigint DEFAULT 0 NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_level_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_level_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_level_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_level_stats_submitted_check CHECK ((submitted >= 0)),
    CONSTRAINT submission_daily_level_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE aredl.submission_daily_reviewer_stats (
    day date NOT NULL,
    reviewer_id uuid NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_reviewer_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE aredl.submission_daily_total_stats (
    day date NOT NULL,
    submitted bigint DEFAULT 0 NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_total_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_total_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_total_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_total_stats_submitted_check CHECK ((submitted >= 0)),
    CONSTRAINT submission_daily_total_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE aredl.submission_history (
    id uuid NOT NULL,
    submission_id uuid NOT NULL,
    reviewer_notes text,
    status public.submission_status NOT NULL,
    timestamp timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    user_notes text,
    reviewer_id uuid,
    mobile boolean,
    custom_copy_id integer,
    video_url character varying,
    raw_url character varying,
    mod_menu character varying,
    priority boolean,
    private_reviewer_notes text,
    locked boolean
);

CREATE TABLE aredl.submissions_enabled (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    enabled boolean NOT NULL,
    moderator uuid NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE arepl.levels (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    position integer,
    name character varying NOT NULL,
    publisher_id uuid NOT NULL,
    points integer DEFAULT 0 NOT NULL,
    level_id integer NOT NULL,
    two_player boolean NOT NULL,
    tags text[] DEFAULT '{}'::text[] NOT NULL,
    description character varying,
    song integer,
    edel_enjoyment double precision,
    is_edel_pending boolean DEFAULT false NOT NULL,
    gddl_tier double precision,
    nlw_tier character varying,
    status public.level_status NOT NULL,
    requires_raw_footage boolean DEFAULT false NOT NULL,
    nlw_tier_estimate character varying,
    CONSTRAINT aredl_levels_level_id_check CHECK ((level_id > 0)),
    CONSTRAINT arepl_levels_status_position_check CHECK ((((status = ANY (ARRAY['Pending'::public.level_status, 'Removed'::public.level_status])) AND (position IS NULL)) OR ((status = ANY (ARRAY['MainList'::public.level_status, 'Legacy'::public.level_status])) AND (position IS NOT NULL))))
);

CREATE TABLE arepl.position_history_full_view (
    ord integer NOT NULL,
    affected_level uuid NOT NULL,
    position integer,
    moved boolean NOT NULL,
    status public.level_status NOT NULL,
    action_at timestamp with time zone NOT NULL,
    cause uuid NOT NULL,
    pos_diff integer
);

CREATE TABLE arepl.records (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    level_id uuid NOT NULL,
    submitted_by uuid NOT NULL,
    mobile boolean DEFAULT false NOT NULL,
    video_url character varying NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    updated_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    is_verification boolean DEFAULT false NOT NULL,
    completion_time bigint DEFAULT 0 NOT NULL,
    hide_video boolean DEFAULT false NOT NULL,
    submission_id uuid NOT NULL,
    achieved_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE arepl.bounties (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    bounty_type public.bounty_type NOT NULL,
    bounty_difficulty public.bounty_difficulty NOT NULL,
    start_date timestamp with time zone NOT NULL,
    end_date timestamp with time zone,
    target_submissions integer,
    is_target_public boolean DEFAULT false NOT NULL
);

CREATE TABLE arepl.bounty_completed (
    user_id uuid NOT NULL,
    bounty_id uuid NOT NULL,
    completed_at timestamp with time zone DEFAULT now() NOT NULL
);

CREATE TABLE arepl.levels_created (
    level_id uuid NOT NULL,
    user_id uuid NOT NULL
);

CREATE TABLE arepl.pack_levels (
    pack_id uuid NOT NULL,
    level_id uuid NOT NULL
);

CREATE TABLE arepl.last_gddl_update (
    id uuid NOT NULL,
    updated_at timestamp with time zone DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE TABLE arepl.level_custom_copies (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    copy_id integer NOT NULL,
    added_by uuid NOT NULL,
    description character varying,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    id_type arepl.custom_id_type NOT NULL,
    status arepl.custom_id_status NOT NULL,
    CONSTRAINT level_ldms_ldm_id_check CHECK ((copy_id > 0))
);

CREATE TABLE arepl.level_notes (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    note text NOT NULL,
    note_type arepl.level_notes_type NOT NULL,
    timestamp timestamp with time zone,
    added_by uuid NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE arepl.level_updates (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    level_id uuid NOT NULL,
    changelog text,
    update_type arepl.level_update_type NOT NULL,
    timestamp timestamp with time zone NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

CREATE TABLE arepl.pack_tiers (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying NOT NULL,
    color character varying NOT NULL,
    placement integer DEFAULT 0 NOT NULL
);

CREATE TABLE arepl.packs (
    id uuid DEFAULT public.uuid_generate_v4() NOT NULL,
    name character varying NOT NULL,
    tier uuid NOT NULL
);

CREATE TABLE arepl.position_history (
    i integer NOT NULL,
    new_position integer,
    old_position integer,
    affected_level uuid NOT NULL,
    level_above uuid,
    level_below uuid,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    old_status public.level_status,
    new_status public.level_status NOT NULL
);

CREATE SEQUENCE arepl.position_history_i_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1;

ALTER SEQUENCE arepl.position_history_i_seq OWNED BY arepl.position_history.i;

CREATE TABLE arepl.submission_daily_level_stats (
    day date NOT NULL,
    level_id uuid NOT NULL,
    submitted bigint DEFAULT 0 NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_level_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_level_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_level_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_level_stats_submitted_check CHECK ((submitted >= 0)),
    CONSTRAINT submission_daily_level_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE arepl.submission_daily_reviewer_stats (
    day date NOT NULL,
    reviewer_id uuid NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_reviewer_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_reviewer_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE arepl.submission_daily_total_stats (
    day date NOT NULL,
    submitted bigint DEFAULT 0 NOT NULL,
    accepted bigint DEFAULT 0 NOT NULL,
    denied bigint DEFAULT 0 NOT NULL,
    under_consideration bigint DEFAULT 0 NOT NULL,
    reviewed bigint DEFAULT 0 NOT NULL,
    CONSTRAINT submission_daily_total_stats_accepted_check CHECK ((accepted >= 0)),
    CONSTRAINT submission_daily_total_stats_denied_check CHECK ((denied >= 0)),
    CONSTRAINT submission_daily_total_stats_reviewed_check CHECK ((reviewed >= 0)),
    CONSTRAINT submission_daily_total_stats_submitted_check CHECK ((submitted >= 0)),
    CONSTRAINT submission_daily_total_stats_under_consideration_check CHECK ((under_consideration >= 0))
);

CREATE TABLE arepl.submission_history (
    id uuid NOT NULL,
    submission_id uuid NOT NULL,
    reviewer_notes text,
    status public.submission_status NOT NULL,
    timestamp timestamp with time zone DEFAULT clock_timestamp() NOT NULL,
    user_notes text,
    reviewer_id uuid,
    mobile boolean,
    custom_copy_id integer,
    video_url character varying,
    raw_url character varying,
    mod_menu character varying,
    priority boolean,
    private_reviewer_notes text,
    locked boolean,
    completion_time bigint
);

CREATE TABLE arepl.submissions_enabled (
    id uuid DEFAULT gen_random_uuid() NOT NULL,
    enabled boolean NOT NULL,
    moderator uuid NOT NULL,
    created_at timestamp with time zone DEFAULT clock_timestamp() NOT NULL
);

ALTER TABLE aredl.position_history ALTER COLUMN i SET DEFAULT nextval('aredl.position_history_i_seq'::regclass);

ALTER TABLE arepl.position_history ALTER COLUMN i SET DEFAULT nextval('arepl.position_history_i_seq'::regclass);


INSERT INTO aredl.bounties (id, level_id, bounty_type, bounty_difficulty, start_date, end_date, target_submissions, is_target_public)
SELECT id, level_id, bounty_type, bounty_difficulty, start_date, end_date, target_submissions, is_target_public
FROM public.bounties
WHERE list_id = 1;

INSERT INTO aredl.bounty_completed (user_id, bounty_id, completed_at)
SELECT user_id, bounty_id, completed_at
FROM public.bounty_completed
WHERE list_id = 1;

INSERT INTO aredl.last_gddl_update (id, updated_at)
SELECT id, updated_at
FROM public.last_gddl_update
WHERE list_id = 1;

INSERT INTO aredl.level_custom_copies (id, level_id, copy_id, added_by, description, created_at, id_type, status)
SELECT id, level_id, copy_id, added_by, description, created_at, id_type::text::aredl.custom_id_type, status::text::aredl.custom_id_status
FROM public.level_custom_copies
WHERE list_id = 1;

INSERT INTO aredl.level_notes (id, level_id, note, note_type, "timestamp", added_by, created_at)
SELECT id, level_id, note, note_type::text::aredl.level_notes_type, "timestamp", added_by, created_at
FROM public.level_notes
WHERE list_id = 1;

INSERT INTO aredl.level_updates (id, level_id, changelog, update_type, "timestamp", created_at)
SELECT id, level_id, changelog, update_type::text::aredl.level_update_type, "timestamp", created_at
FROM public.level_updates
WHERE list_id = 1;

INSERT INTO aredl.levels (id, "position", name, publisher_id, points, level_id, two_player, tags, description, song, edel_enjoyment, is_edel_pending, gddl_tier, nlw_tier, status, requires_raw_footage, nlw_tier_estimate)
SELECT id, "position", name, publisher_id, points, level_id, two_player, tags, description, song, edel_enjoyment, is_edel_pending, gddl_tier, nlw_tier, status, requires_raw_footage, nlw_tier_estimate
FROM public.levels
WHERE list_id = 1;

INSERT INTO aredl.levels_created (level_id, user_id)
SELECT level_id, user_id
FROM public.levels_created
WHERE list_id = 1;

INSERT INTO aredl.pack_levels (pack_id, level_id)
SELECT pack_id, level_id
FROM public.pack_levels
WHERE list_id = 1;

INSERT INTO aredl.pack_tiers (id, name, color, placement)
SELECT id, name, color, placement
FROM public.pack_tiers
WHERE list_id = 1;

INSERT INTO aredl.packs (id, name, tier)
SELECT id, name, tier
FROM public.packs
WHERE list_id = 1;

INSERT INTO aredl.position_history (i, new_position, old_position, affected_level, level_above, level_below, created_at, old_status, new_status)
SELECT i, new_position, old_position, affected_level, level_above, level_below, created_at, old_status, new_status
FROM public.position_history
WHERE list_id = 1;

INSERT INTO aredl.position_history_full_view (ord, affected_level, "position", moved, status, action_at, cause, pos_diff)
SELECT ord, affected_level, "position", moved, status, action_at, cause, pos_diff
FROM public.position_history_full_view
WHERE list_id = 1;

INSERT INTO aredl.records (id, level_id, submitted_by, mobile, video_url, created_at, updated_at, is_verification, hide_video, submission_id, achieved_at)
SELECT id, level_id, submitted_by, mobile, video_url, created_at, updated_at, is_verification, hide_video, submission_id, achieved_at
FROM public.records
WHERE list_id = 1;

INSERT INTO aredl.submission_daily_level_stats (day, level_id, submitted, accepted, denied, under_consideration, reviewed)
SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_level_stats
WHERE list_id = 1;

INSERT INTO aredl.submission_daily_reviewer_stats (day, reviewer_id, accepted, denied, under_consideration, reviewed)
SELECT day, reviewer_id, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_reviewer_stats
WHERE list_id = 1;

INSERT INTO aredl.submission_daily_total_stats (day, submitted, accepted, denied, under_consideration, reviewed)
SELECT day, submitted, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_total_stats
WHERE list_id = 1;

INSERT INTO aredl.submission_history (id, submission_id, reviewer_notes, status, "timestamp", user_notes, reviewer_id, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, private_reviewer_notes, locked)
SELECT id, submission_id, reviewer_notes, status, "timestamp", user_notes, reviewer_id, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, private_reviewer_notes, locked
FROM public.submission_history
WHERE list_id = 1;

INSERT INTO aredl.submissions (id, level_id, submitted_by, mobile, custom_copy_id, video_url, raw_url, reviewer_id, priority, reviewer_notes, user_notes, created_at, status, mod_menu, updated_at, private_reviewer_notes, locked, priority_at)
SELECT id, level_id, submitted_by, mobile, custom_copy_id, video_url, raw_url, reviewer_id, priority, reviewer_notes, user_notes, created_at, status, mod_menu, updated_at, private_reviewer_notes, locked, priority_at
FROM public.submissions
WHERE list_id = 1;

INSERT INTO aredl.submissions_enabled (id, enabled, moderator, created_at)
SELECT id, enabled, moderator, created_at
FROM public.submissions_enabled
WHERE list_id = 1;

SELECT setval('aredl.position_history_i_seq',
              GREATEST(COALESCE(MAX(i), 0), 1), MAX(i) IS NOT NULL)
FROM aredl.position_history;

INSERT INTO arepl.bounties (id, level_id, bounty_type, bounty_difficulty, start_date, end_date, target_submissions, is_target_public)
SELECT id, level_id, bounty_type, bounty_difficulty, start_date, end_date, target_submissions, is_target_public
FROM public.bounties
WHERE list_id = 2;

INSERT INTO arepl.bounty_completed (user_id, bounty_id, completed_at)
SELECT user_id, bounty_id, completed_at
FROM public.bounty_completed
WHERE list_id = 2;

INSERT INTO arepl.last_gddl_update (id, updated_at)
SELECT id, updated_at
FROM public.last_gddl_update
WHERE list_id = 2;

INSERT INTO arepl.level_custom_copies (id, level_id, copy_id, added_by, description, created_at, id_type, status)
SELECT id, level_id, copy_id, added_by, description, created_at, id_type::text::arepl.custom_id_type, status::text::arepl.custom_id_status
FROM public.level_custom_copies
WHERE list_id = 2;

INSERT INTO arepl.level_notes (id, level_id, note, note_type, "timestamp", added_by, created_at)
SELECT id, level_id, note, note_type::text::arepl.level_notes_type, "timestamp", added_by, created_at
FROM public.level_notes
WHERE list_id = 2;

INSERT INTO arepl.level_updates (id, level_id, changelog, update_type, "timestamp", created_at)
SELECT id, level_id, changelog, update_type::text::arepl.level_update_type, "timestamp", created_at
FROM public.level_updates
WHERE list_id = 2;

INSERT INTO arepl.levels (id, "position", name, publisher_id, points, level_id, two_player, tags, description, song, edel_enjoyment, is_edel_pending, gddl_tier, nlw_tier, status, requires_raw_footage, nlw_tier_estimate)
SELECT id, "position", name, publisher_id, points, level_id, two_player, tags, description, song, edel_enjoyment, is_edel_pending, gddl_tier, nlw_tier, status, requires_raw_footage, nlw_tier_estimate
FROM public.levels
WHERE list_id = 2;

INSERT INTO arepl.levels_created (level_id, user_id)
SELECT level_id, user_id
FROM public.levels_created
WHERE list_id = 2;

INSERT INTO arepl.pack_levels (pack_id, level_id)
SELECT pack_id, level_id
FROM public.pack_levels
WHERE list_id = 2;

INSERT INTO arepl.pack_tiers (id, name, color, placement)
SELECT id, name, color, placement
FROM public.pack_tiers
WHERE list_id = 2;

INSERT INTO arepl.packs (id, name, tier)
SELECT id, name, tier
FROM public.packs
WHERE list_id = 2;

INSERT INTO arepl.position_history (i, new_position, old_position, affected_level, level_above, level_below, created_at, old_status, new_status)
SELECT i, new_position, old_position, affected_level, level_above, level_below, created_at, old_status, new_status
FROM public.position_history
WHERE list_id = 2;

INSERT INTO arepl.position_history_full_view (ord, affected_level, "position", moved, status, action_at, cause, pos_diff)
SELECT ord, affected_level, "position", moved, status, action_at, cause, pos_diff
FROM public.position_history_full_view
WHERE list_id = 2;

INSERT INTO arepl.records (id, level_id, submitted_by, mobile, video_url, created_at, updated_at, is_verification, completion_time, hide_video, submission_id, achieved_at)
SELECT id, level_id, submitted_by, mobile, video_url, created_at, updated_at, is_verification, completion_time, hide_video, submission_id, achieved_at
FROM public.records
WHERE list_id = 2;

INSERT INTO arepl.submission_daily_level_stats (day, level_id, submitted, accepted, denied, under_consideration, reviewed)
SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_level_stats
WHERE list_id = 2;

INSERT INTO arepl.submission_daily_reviewer_stats (day, reviewer_id, accepted, denied, under_consideration, reviewed)
SELECT day, reviewer_id, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_reviewer_stats
WHERE list_id = 2;

INSERT INTO arepl.submission_daily_total_stats (day, submitted, accepted, denied, under_consideration, reviewed)
SELECT day, submitted, accepted, denied, under_consideration, reviewed
FROM public.submission_daily_total_stats
WHERE list_id = 2;

INSERT INTO arepl.submission_history (id, submission_id, reviewer_notes, status, "timestamp", user_notes, reviewer_id, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, private_reviewer_notes, locked, completion_time)
SELECT id, submission_id, reviewer_notes, status, "timestamp", user_notes, reviewer_id, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, private_reviewer_notes, locked, completion_time
FROM public.submission_history
WHERE list_id = 2;

INSERT INTO arepl.submissions (id, level_id, submitted_by, mobile, custom_copy_id, video_url, raw_url, reviewer_id, priority, reviewer_notes, user_notes, created_at, status, mod_menu, updated_at, completion_time, private_reviewer_notes, locked, priority_at)
SELECT id, level_id, submitted_by, mobile, custom_copy_id, video_url, raw_url, reviewer_id, priority, reviewer_notes, user_notes, created_at, status, mod_menu, updated_at, completion_time, private_reviewer_notes, locked, priority_at
FROM public.submissions
WHERE list_id = 2;

INSERT INTO arepl.submissions_enabled (id, enabled, moderator, created_at)
SELECT id, enabled, moderator, created_at
FROM public.submissions_enabled
WHERE list_id = 2;

SELECT setval('arepl.position_history_i_seq',
              GREATEST(COALESCE(MAX(i), 0), 1), MAX(i) IS NOT NULL)
FROM arepl.position_history;


ALTER TABLE aredl.bounties
    ADD CONSTRAINT bounties_pkey PRIMARY KEY (id);

ALTER TABLE aredl.bounty_completed
    ADD CONSTRAINT bounty_completed_pkey PRIMARY KEY (user_id, bounty_id);

ALTER TABLE aredl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_pkey PRIMARY KEY (id);

ALTER TABLE aredl.level_custom_copies
    ADD CONSTRAINT level_ldms_pkey PRIMARY KEY (id);

ALTER TABLE aredl.level_notes
    ADD CONSTRAINT level_notes_pkey PRIMARY KEY (id);

ALTER TABLE aredl.level_updates
    ADD CONSTRAINT level_updates_pkey PRIMARY KEY (id);

ALTER TABLE aredl.levels_created
    ADD CONSTRAINT levels_created_pkey PRIMARY KEY (level_id, user_id);

ALTER TABLE aredl.levels
    ADD CONSTRAINT levels_level_id_two_player_key UNIQUE (level_id, two_player);

ALTER TABLE aredl.levels
    ADD CONSTRAINT levels_pkey PRIMARY KEY (id);

ALTER TABLE aredl.pack_levels
    ADD CONSTRAINT pack_levels_pkey PRIMARY KEY (pack_id, level_id);

ALTER TABLE aredl.pack_tiers
    ADD CONSTRAINT pack_tiers_pkey PRIMARY KEY (id);

ALTER TABLE aredl.packs
    ADD CONSTRAINT packs_pkey PRIMARY KEY (id);

ALTER TABLE aredl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_pkey PRIMARY KEY (ord, affected_level);

ALTER TABLE aredl.position_history
    ADD CONSTRAINT position_history_pkey PRIMARY KEY (i);

ALTER TABLE aredl.records
    ADD CONSTRAINT records_level_id_submitted_by_key UNIQUE (level_id, submitted_by);

ALTER TABLE aredl.records
    ADD CONSTRAINT records_pkey PRIMARY KEY (id);

ALTER TABLE aredl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_pkey PRIMARY KEY (day, level_id);

ALTER TABLE aredl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_pkey PRIMARY KEY (day, reviewer_id);

ALTER TABLE aredl.submission_daily_total_stats
    ADD CONSTRAINT submission_daily_total_stats_pkey PRIMARY KEY (day);

ALTER TABLE aredl.submission_history
    ADD CONSTRAINT submission_history_pkey PRIMARY KEY (id);

ALTER TABLE aredl.submissions_enabled
    ADD CONSTRAINT submissions_enabled_pkey PRIMARY KEY (id);

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_level_id_submitted_by_key UNIQUE (level_id, submitted_by);

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_pkey PRIMARY KEY (id);

ALTER TABLE arepl.bounties
    ADD CONSTRAINT bounties_pkey PRIMARY KEY (id);

ALTER TABLE arepl.bounty_completed
    ADD CONSTRAINT bounty_completed_pkey PRIMARY KEY (user_id, bounty_id);

ALTER TABLE arepl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_pkey PRIMARY KEY (id);

ALTER TABLE arepl.level_custom_copies
    ADD CONSTRAINT level_ldms_pkey PRIMARY KEY (id);

ALTER TABLE arepl.level_notes
    ADD CONSTRAINT level_notes_pkey PRIMARY KEY (id);

ALTER TABLE arepl.level_updates
    ADD CONSTRAINT level_updates_pkey PRIMARY KEY (id);

ALTER TABLE arepl.levels_created
    ADD CONSTRAINT levels_created_pkey PRIMARY KEY (level_id, user_id);

ALTER TABLE arepl.levels
    ADD CONSTRAINT levels_level_id_two_player_key UNIQUE (level_id, two_player);

ALTER TABLE arepl.levels
    ADD CONSTRAINT levels_pkey PRIMARY KEY (id);

ALTER TABLE arepl.pack_levels
    ADD CONSTRAINT pack_levels_pkey PRIMARY KEY (pack_id, level_id);

ALTER TABLE arepl.pack_tiers
    ADD CONSTRAINT pack_tiers_pkey PRIMARY KEY (id);

ALTER TABLE arepl.packs
    ADD CONSTRAINT packs_pkey PRIMARY KEY (id);

ALTER TABLE arepl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_pkey PRIMARY KEY (ord, affected_level);

ALTER TABLE arepl.position_history
    ADD CONSTRAINT position_history_pkey PRIMARY KEY (i);

ALTER TABLE arepl.records
    ADD CONSTRAINT records_level_id_submitted_by_key UNIQUE (level_id, submitted_by);

ALTER TABLE arepl.records
    ADD CONSTRAINT records_pkey PRIMARY KEY (id);

ALTER TABLE arepl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_pkey PRIMARY KEY (day, level_id);

ALTER TABLE arepl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_pkey PRIMARY KEY (day, reviewer_id);

ALTER TABLE arepl.submission_daily_total_stats
    ADD CONSTRAINT submission_daily_total_stats_pkey PRIMARY KEY (day);

ALTER TABLE arepl.submission_history
    ADD CONSTRAINT submission_history_pkey PRIMARY KEY (id);

ALTER TABLE arepl.submissions_enabled
    ADD CONSTRAINT submissions_enabled_pkey PRIMARY KEY (id);

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_level_id_submitted_by_key UNIQUE (level_id, submitted_by);

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_pkey PRIMARY KEY (id);

ALTER TABLE aredl.submission_history
    ADD CONSTRAINT aredl_submission_history_submission_fk FOREIGN KEY (submission_id) REFERENCES aredl.submissions(id) ON DELETE CASCADE;

ALTER TABLE aredl.bounties
    ADD CONSTRAINT bounties_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON DELETE CASCADE;

ALTER TABLE aredl.bounty_completed
    ADD CONSTRAINT bounty_completed_bounty_id_fkey FOREIGN KEY (bounty_id) REFERENCES aredl.bounties(id) ON DELETE CASCADE;

ALTER TABLE aredl.bounty_completed
    ADD CONSTRAINT bounty_completed_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE aredl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_id_fkey FOREIGN KEY (id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.level_custom_copies
    ADD CONSTRAINT level_ldms_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.level_custom_copies
    ADD CONSTRAINT level_ldms_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.level_notes
    ADD CONSTRAINT level_notes_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.level_notes
    ADD CONSTRAINT level_notes_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.level_updates
    ADD CONSTRAINT level_updates_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.levels_created
    ADD CONSTRAINT levels_created_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.levels_created
    ADD CONSTRAINT levels_created_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.levels
    ADD CONSTRAINT levels_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.pack_levels
    ADD CONSTRAINT pack_levels_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.pack_levels
    ADD CONSTRAINT pack_levels_pack_id_fkey FOREIGN KEY (pack_id) REFERENCES aredl.packs(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.packs
    ADD CONSTRAINT packs_tier_fkey FOREIGN KEY (tier) REFERENCES aredl.pack_tiers(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.position_history
    ADD CONSTRAINT position_history_affected_level_fkey FOREIGN KEY (affected_level) REFERENCES aredl.levels(id);

ALTER TABLE aredl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_affected_level_fkey FOREIGN KEY (affected_level) REFERENCES aredl.levels(id);

ALTER TABLE aredl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_cause_fkey FOREIGN KEY (cause) REFERENCES aredl.levels(id);

ALTER TABLE aredl.position_history
    ADD CONSTRAINT position_history_level_above_fkey FOREIGN KEY (level_above) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.position_history
    ADD CONSTRAINT position_history_level_below_fkey FOREIGN KEY (level_below) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.records
    ADD CONSTRAINT records_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.records
    ADD CONSTRAINT records_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES aredl.submissions(id) ON DELETE CASCADE;

ALTER TABLE aredl.records
    ADD CONSTRAINT records_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON UPDATE CASCADE;

ALTER TABLE aredl.submission_history
    ADD CONSTRAINT submission_history_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON UPDATE CASCADE;

ALTER TABLE aredl.submissions_enabled
    ADD CONSTRAINT submissions_enabled_moderator_fkey FOREIGN KEY (moderator) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_level_id_fkey FOREIGN KEY (level_id) REFERENCES aredl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.submission_history
    ADD CONSTRAINT arepl_submission_history_submission_fk FOREIGN KEY (submission_id) REFERENCES arepl.submissions(id) ON DELETE CASCADE;

ALTER TABLE arepl.bounties
    ADD CONSTRAINT bounties_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON DELETE CASCADE;

ALTER TABLE arepl.bounty_completed
    ADD CONSTRAINT bounty_completed_bounty_id_fkey FOREIGN KEY (bounty_id) REFERENCES arepl.bounties(id) ON DELETE CASCADE;

ALTER TABLE arepl.bounty_completed
    ADD CONSTRAINT bounty_completed_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON DELETE CASCADE;

ALTER TABLE arepl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_id_fkey FOREIGN KEY (id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.level_custom_copies
    ADD CONSTRAINT level_ldms_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.level_custom_copies
    ADD CONSTRAINT level_ldms_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.level_notes
    ADD CONSTRAINT level_notes_added_by_fkey FOREIGN KEY (added_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.level_notes
    ADD CONSTRAINT level_notes_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.level_updates
    ADD CONSTRAINT level_updates_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.levels_created
    ADD CONSTRAINT levels_created_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.levels_created
    ADD CONSTRAINT levels_created_user_id_fkey FOREIGN KEY (user_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.levels
    ADD CONSTRAINT levels_publisher_id_fkey FOREIGN KEY (publisher_id) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.pack_levels
    ADD CONSTRAINT pack_levels_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.pack_levels
    ADD CONSTRAINT pack_levels_pack_id_fkey FOREIGN KEY (pack_id) REFERENCES arepl.packs(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.packs
    ADD CONSTRAINT packs_tier_fkey FOREIGN KEY (tier) REFERENCES arepl.pack_tiers(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.position_history
    ADD CONSTRAINT position_history_affected_level_fkey FOREIGN KEY (affected_level) REFERENCES arepl.levels(id);

ALTER TABLE arepl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_affected_level_fkey FOREIGN KEY (affected_level) REFERENCES arepl.levels(id);

ALTER TABLE arepl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_cause_fkey FOREIGN KEY (cause) REFERENCES arepl.levels(id);

ALTER TABLE arepl.position_history
    ADD CONSTRAINT position_history_level_above_fkey FOREIGN KEY (level_above) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.position_history
    ADD CONSTRAINT position_history_level_below_fkey FOREIGN KEY (level_below) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.records
    ADD CONSTRAINT records_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.records
    ADD CONSTRAINT records_submission_id_fkey FOREIGN KEY (submission_id) REFERENCES arepl.submissions(id) ON DELETE CASCADE;

ALTER TABLE arepl.records
    ADD CONSTRAINT records_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON UPDATE CASCADE;

ALTER TABLE arepl.submission_history
    ADD CONSTRAINT submission_history_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON UPDATE CASCADE;

ALTER TABLE arepl.submissions_enabled
    ADD CONSTRAINT submissions_enabled_moderator_fkey FOREIGN KEY (moderator) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE SET NULL;

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_level_id_fkey FOREIGN KEY (level_id) REFERENCES arepl.levels(id) ON UPDATE CASCADE ON DELETE CASCADE;

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_reviewer_id_fkey FOREIGN KEY (reviewer_id) REFERENCES public.users(id) ON DELETE SET NULL;

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_submitted_by_fkey FOREIGN KEY (submitted_by) REFERENCES public.users(id) ON UPDATE CASCADE ON DELETE CASCADE;


CREATE INDEX aredl_hist_rev_ts_idx ON aredl.submission_history (reviewer_id, timestamp);
CREATE INDEX aredl_hist_sub_ts_id_idx ON aredl.submission_history (submission_id, timestamp, id);
CREATE INDEX aredl_hist_ts_idx ON aredl.submission_history (timestamp);
CREATE INDEX aredl_levels_created_user_level_idx ON aredl.levels_created (user_id, level_id);
CREATE INDEX aredl_levels_position_idx ON aredl.levels (position) WHERE (position IS NOT NULL);
CREATE INDEX aredl_levels_publisher_position_idx ON aredl.levels (publisher_id, position);
CREATE INDEX aredl_pack_levels_level_pack_idx ON aredl.pack_levels (level_id, pack_id);
CREATE INDEX aredl_position_history_full_view_affected_ord_idx ON aredl.position_history_full_view (affected_level, ord DESC);
CREATE INDEX aredl_position_history_full_view_level_action_idx ON aredl.position_history_full_view (affected_level, action_at DESC, ord DESC) INCLUDE (position, status);
CREATE INDEX aredl_position_history_full_view_level_first_placed_idx ON aredl.position_history_full_view (affected_level, action_at, ord) INCLUDE (position, status) WHERE (position IS NOT NULL);
CREATE INDEX aredl_records_first_victor_idx ON aredl.records (level_id, achieved_at, created_at, id) INCLUDE (submitted_by) WHERE (is_verification = false);
CREATE INDEX aredl_records_submission_id_idx ON aredl.records (submission_id);
CREATE INDEX aredl_records_submitted_by_level_idx ON aredl.records (submitted_by, level_id);
CREATE INDEX aredl_submission_daily_level_stats_level_day_idx ON aredl.submission_daily_level_stats (level_id, day DESC);
CREATE INDEX aredl_submission_daily_reviewer_stats_reviewer_day_idx ON aredl.submission_daily_reviewer_stats (reviewer_id, day DESC);
CREATE INDEX aredl_submissions_pending_priority_created_idx ON aredl.submissions (priority, created_at) WHERE (status = 'Pending'::public.submission_status);
CREATE INDEX aredl_submissions_pending_priority_updated_idx ON aredl.submissions (priority, updated_at) WHERE (status = 'Pending'::public.submission_status);
CREATE INDEX aredl_submissions_pending_updated_idx ON aredl.submissions (updated_at) WHERE (status = 'Pending'::public.submission_status);
CREATE INDEX aredl_submissions_status_idx ON aredl.submissions (status);
CREATE INDEX idx_bounty_completed_bounty_id ON aredl.bounty_completed (bounty_id);
CREATE INDEX idx_bounty_completed_user_id ON aredl.bounty_completed (user_id);
CREATE INDEX arepl_hist_rev_ts_idx ON arepl.submission_history (reviewer_id, timestamp);
CREATE INDEX arepl_hist_sub_ts_id_idx ON arepl.submission_history (submission_id, timestamp, id);
CREATE INDEX arepl_hist_ts_idx ON arepl.submission_history (timestamp);
CREATE INDEX arepl_levels_created_user_level_idx ON arepl.levels_created (user_id, level_id);
CREATE INDEX arepl_levels_position_idx ON arepl.levels (position) WHERE (position IS NOT NULL);
CREATE INDEX arepl_levels_publisher_position_idx ON arepl.levels (publisher_id, position);
CREATE INDEX arepl_pack_levels_level_pack_idx ON arepl.pack_levels (level_id, pack_id);
CREATE INDEX arepl_position_history_full_view_affected_ord_idx ON arepl.position_history_full_view (affected_level, ord DESC);
CREATE INDEX arepl_position_history_full_view_level_action_idx ON arepl.position_history_full_view (affected_level, action_at DESC, ord DESC) INCLUDE (position, status);
CREATE INDEX arepl_position_history_full_view_level_first_placed_idx ON arepl.position_history_full_view (affected_level, action_at, ord) INCLUDE (position, status) WHERE (position IS NOT NULL);
CREATE INDEX arepl_records_fastest_time_idx ON arepl.records (level_id, completion_time, achieved_at, id) INCLUDE (submitted_by) WHERE (is_verification = false);
CREATE INDEX arepl_records_first_victor_idx ON arepl.records (level_id, achieved_at, created_at, id) INCLUDE (submitted_by) WHERE (is_verification = false);
CREATE INDEX arepl_records_submitted_by_level_idx ON arepl.records (submitted_by, level_id);
CREATE INDEX arepl_submission_daily_level_stats_level_day_idx ON arepl.submission_daily_level_stats (level_id, day DESC);
CREATE INDEX arepl_submission_daily_reviewer_stats_reviewer_day_idx ON arepl.submission_daily_reviewer_stats (reviewer_id, day DESC);
CREATE INDEX arepl_submissions_status_idx ON arepl.submissions (status);
CREATE INDEX idx_bounty_completed_bounty_id ON arepl.bounty_completed (bounty_id);
CREATE INDEX idx_bounty_completed_user_id ON arepl.bounty_completed (user_id);

CREATE FUNCTION aredl.append_position_history_full_view(history_entry_i integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    history_ord INTEGER;
    current_ord INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    LOCK TABLE aredl.position_history_full_view IN EXCLUSIVE MODE;

    SELECT COUNT(*)::INTEGER
    INTO history_ord
    FROM aredl.position_history
    WHERE i <= history_entry_i;

    IF history_ord = 0 THEN
        RAISE EXCEPTION 'Position history entry % does not exist', history_entry_i;
    END IF;

    SELECT COALESCE(MAX(ord), 0)
    INTO current_ord
    FROM aredl.position_history_full_view;

    IF history_ord <> current_ord + 1 THEN
        RAISE EXCEPTION 'Cannot append position history ord %, current ord is %', history_ord, current_ord;
    END IF;

    INSERT INTO aredl.position_history_full_view (
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
        FROM aredl.position_history ph
        WHERE ph.i = history_entry_i
    ),
    previous_state AS (
        SELECT DISTINCT ON (phv.affected_level)
            phv.affected_level,
            phv.position,
            phv.status
        FROM aredl.position_history_full_view phv
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
        f.ord,
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
        FROM aredl.position_history_full_view phv
        WHERE phv.affected_level = f.affected_level
        ORDER BY phv.ord DESC
        LIMIT 1
    ) prev ON true;
END;
$$;

CREATE FUNCTION aredl.apply_submission_daily_stats_diff(p_day date, p_reviewer_id uuid, p_level_id uuid, p_submitted bigint, p_accepted bigint, p_denied bigint, p_under_consideration bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF p_submitted = 0
        AND p_accepted = 0
        AND p_denied = 0
        AND p_under_consideration = 0
    THEN
        RETURN;
    END IF;

    INSERT INTO aredl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day) DO UPDATE
    SET
        submitted = aredl.submission_daily_total_stats.submitted + EXCLUDED.submitted,
        accepted = aredl.submission_daily_total_stats.accepted + EXCLUDED.accepted,
        denied = aredl.submission_daily_total_stats.denied + EXCLUDED.denied,
        under_consideration = aredl.submission_daily_total_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = aredl.submission_daily_total_stats.reviewed + EXCLUDED.reviewed;

    INSERT INTO aredl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_level_id,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day, level_id) DO UPDATE
    SET
        submitted = aredl.submission_daily_level_stats.submitted + EXCLUDED.submitted,
        accepted = aredl.submission_daily_level_stats.accepted + EXCLUDED.accepted,
        denied = aredl.submission_daily_level_stats.denied + EXCLUDED.denied,
        under_consideration = aredl.submission_daily_level_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = aredl.submission_daily_level_stats.reviewed + EXCLUDED.reviewed;

    IF p_reviewer_id IS NOT NULL
        AND (
            p_accepted <> 0
            OR p_denied <> 0
            OR p_under_consideration <> 0
        )
    THEN
        INSERT INTO aredl.submission_daily_reviewer_stats (
            day,
            reviewer_id,
            accepted,
            denied,
            under_consideration,
            reviewed
        )
        VALUES (
            p_day,
            p_reviewer_id,
            p_accepted,
            p_denied,
            p_under_consideration,
            p_accepted + p_denied + p_under_consideration
        )
        ON CONFLICT (day, reviewer_id) DO UPDATE
        SET
            accepted = aredl.submission_daily_reviewer_stats.accepted + EXCLUDED.accepted,
            denied = aredl.submission_daily_reviewer_stats.denied + EXCLUDED.denied,
            under_consideration = aredl.submission_daily_reviewer_stats.under_consideration + EXCLUDED.under_consideration,
            reviewed = aredl.submission_daily_reviewer_stats.reviewed + EXCLUDED.reviewed;
    END IF;

END;
$$;

CREATE FUNCTION aredl.level_move() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE aredl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE id <> NEW.id
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
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
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

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM aredl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$;

CREATE FUNCTION aredl.level_place() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        RETURN NULL;
    END IF;

    UPDATE aredl.levels
    SET position = position + 1
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND position >= NEW.position;

    RETURN NULL;
END;
$$;

CREATE FUNCTION aredl.level_place_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
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

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM aredl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$;

CREATE FUNCTION aredl.levels_points_after_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM aredl.recalculate_points();
    RETURN NULL;
END;
$$;

CREATE FUNCTION aredl.levels_points_before_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN aredl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') + 1 AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$;

CREATE FUNCTION aredl.levels_points_before_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN aredl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') AS INT) + CASE WHEN OLD.status = 'MainList' THEN 0 ELSE 1 END)
        ELSE 0
    END;
    RETURN NEW;
END;
$$;

CREATE FUNCTION aredl.max_list_pos() RETURNS integer
    LANGUAGE sql
    AS $$
    SELECT COALESCE(MAX(position), 0) FROM aredl.levels WHERE status = 'MainList';
$$;

CREATE FUNCTION aredl.max_list_pos_legacy() RETURNS integer
    LANGUAGE sql
    AS $$
    SELECT COALESCE(MAX(position), 0) FROM aredl.levels WHERE status IN ('MainList', 'Legacy');
$$;

CREATE FUNCTION aredl.point_formula(pos integer, level_count integer) RETURNS integer
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION aredl.rebuild_position_history_full_view() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    TRUNCATE TABLE aredl.position_history_full_view;

    INSERT INTO aredl.position_history_full_view (
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
        FROM aredl.position_history
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
    SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff
    FROM filtered;
END;
$$;

CREATE FUNCTION aredl.rebuild_submission_daily_stats() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    TRUNCATE
        aredl.submission_daily_total_stats,
        aredl.submission_daily_reviewer_stats,
        aredl.submission_daily_level_stats;

    INSERT INTO aredl.submission_daily_total_stats (
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
        FROM aredl.submission_history h
        INNER JOIN aredl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status)::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status)::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day
    )
    SELECT day, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;

    INSERT INTO aredl.submission_daily_reviewer_stats (
        day,
        reviewer_id,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    SELECT
        DATE(h.timestamp) AS day,
        h.reviewer_id,
        SUM((h.status = 'Accepted'::submission_status)::int)::bigint AS accepted,
        SUM((h.status = 'Denied'::submission_status)::int)::bigint AS denied,
        SUM((h.status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
        SUM(
            (h.status = 'Accepted'::submission_status)::int
            + (h.status = 'Denied'::submission_status)::int
            + (h.status = 'UnderConsideration'::submission_status)::int
        )::bigint AS reviewed
    FROM aredl.submission_history h
    INNER JOIN aredl.submissions s ON s.id = h.submission_id
    WHERE h.reviewer_id IS NOT NULL
    GROUP BY DATE(h.timestamp), h.reviewer_id
    HAVING SUM((h.status = 'Accepted'::submission_status)::int) <> 0
        OR SUM((h.status = 'Denied'::submission_status)::int) <> 0
        OR SUM((h.status = 'UnderConsideration'::submission_status)::int) <> 0;

    INSERT INTO aredl.submission_daily_level_stats (
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
            s.level_id,
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
        FROM aredl.submission_history h
        INNER JOIN aredl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            level_id,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status)::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status)::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day, level_id
    )
    SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;
END;
$$;

CREATE FUNCTION aredl.recalculate_points() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE aredl.levels
    SET points = CASE
        WHEN status = 'MainList' THEN aredl.point_formula(position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
END;
$$;

CREATE FUNCTION aredl.submission_is_only_claim_toggle(old_submission aredl.submissions, new_submission aredl.submissions) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION aredl.submission_log_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF aredl.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;

        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION aredl.submission_sync_record() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status = 'Accepted' THEN
        INSERT INTO aredl.records AS r (
            level_id,
            submitted_by,
            mobile,
            video_url,
            submission_id
        )
        VALUES (
            NEW.level_id,
            NEW.submitted_by,
            NEW.mobile,
            NEW.video_url,
            NEW.id
        )
        ON CONFLICT (level_id, submitted_by)
        DO UPDATE SET
            mobile = EXCLUDED.mobile,
            video_url = EXCLUDED.video_url;

    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION aredl.submission_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION aredl.update_submission_daily_stats_from_submission() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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

        IF aredl.submission_is_only_claim_toggle(OLD, NEW) THEN
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
    accepted_diff = (NEW.status = 'Accepted'::submission_status)::int;
    denied_diff = (NEW.status = 'Denied'::submission_status)::int;
    under_consideration_diff = (NEW.status = 'UnderConsideration'::submission_status)::int;

    PERFORM aredl.apply_submission_daily_stats_diff(
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
$$;

CREATE FUNCTION aredl.validate_position_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
        highestPos := aredl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := aredl.max_list_pos_legacy() + 1;
        lowestPos := aredl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION aredl.validate_position_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
            highestPos := aredl.max_list_pos();
        ELSE
            highestPos := aredl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := aredl.max_list_pos();
        ELSE
            lowestPos := aredl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := aredl.max_list_pos_legacy();
        ELSE
            highestPos := aredl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.append_position_history_full_view(history_entry_i integer) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    history_ord INTEGER;
    current_ord INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    LOCK TABLE arepl.position_history_full_view IN EXCLUSIVE MODE;

    SELECT COUNT(*)::INTEGER
    INTO history_ord
    FROM arepl.position_history
    WHERE i <= history_entry_i;

    IF history_ord = 0 THEN
        RAISE EXCEPTION 'Position history entry % does not exist', history_entry_i;
    END IF;

    SELECT COALESCE(MAX(ord), 0)
    INTO current_ord
    FROM arepl.position_history_full_view;

    IF history_ord <> current_ord + 1 THEN
        RAISE EXCEPTION 'Cannot append position history ord %, current ord is %', history_ord, current_ord;
    END IF;

    INSERT INTO arepl.position_history_full_view (
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
        FROM arepl.position_history ph
        WHERE ph.i = history_entry_i
    ),
    previous_state AS (
        SELECT DISTINCT ON (phv.affected_level)
            phv.affected_level,
            phv.position,
            phv.status
        FROM arepl.position_history_full_view phv
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
        f.ord,
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
        FROM arepl.position_history_full_view phv
        WHERE phv.affected_level = f.affected_level
        ORDER BY phv.ord DESC
        LIMIT 1
    ) prev ON true;
END;
$$;

CREATE FUNCTION arepl.apply_submission_daily_stats_diff(p_day date, p_reviewer_id uuid, p_level_id uuid, p_submitted bigint, p_accepted bigint, p_denied bigint, p_under_consideration bigint) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF p_submitted = 0
        AND p_accepted = 0
        AND p_denied = 0
        AND p_under_consideration = 0
    THEN
        RETURN;
    END IF;

    INSERT INTO arepl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day) DO UPDATE
    SET
        submitted = arepl.submission_daily_total_stats.submitted + EXCLUDED.submitted,
        accepted = arepl.submission_daily_total_stats.accepted + EXCLUDED.accepted,
        denied = arepl.submission_daily_total_stats.denied + EXCLUDED.denied,
        under_consideration = arepl.submission_daily_total_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = arepl.submission_daily_total_stats.reviewed + EXCLUDED.reviewed;

    INSERT INTO arepl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_level_id,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day, level_id) DO UPDATE
    SET
        submitted = arepl.submission_daily_level_stats.submitted + EXCLUDED.submitted,
        accepted = arepl.submission_daily_level_stats.accepted + EXCLUDED.accepted,
        denied = arepl.submission_daily_level_stats.denied + EXCLUDED.denied,
        under_consideration = arepl.submission_daily_level_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = arepl.submission_daily_level_stats.reviewed + EXCLUDED.reviewed;

    IF p_reviewer_id IS NOT NULL
        AND (
            p_accepted <> 0
            OR p_denied <> 0
            OR p_under_consideration <> 0
        )
    THEN
        INSERT INTO arepl.submission_daily_reviewer_stats (
            day,
            reviewer_id,
            accepted,
            denied,
            under_consideration,
            reviewed
        )
        VALUES (
            p_day,
            p_reviewer_id,
            p_accepted,
            p_denied,
            p_under_consideration,
            p_accepted + p_denied + p_under_consideration
        )
        ON CONFLICT (day, reviewer_id) DO UPDATE
        SET
            accepted = arepl.submission_daily_reviewer_stats.accepted + EXCLUDED.accepted,
            denied = arepl.submission_daily_reviewer_stats.denied + EXCLUDED.denied,
            under_consideration = arepl.submission_daily_reviewer_stats.under_consideration + EXCLUDED.under_consideration,
            reviewed = arepl.submission_daily_reviewer_stats.reviewed + EXCLUDED.reviewed;
    END IF;

END;
$$;

CREATE FUNCTION arepl.level_move() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE arepl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE id <> NEW.id
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
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
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

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM arepl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$;

CREATE FUNCTION arepl.level_place() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        RETURN NULL;
    END IF;

    UPDATE arepl.levels
    SET position = position + 1
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND position >= NEW.position;

    RETURN NULL;
END;
$$;

CREATE FUNCTION arepl.level_place_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
DECLARE
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
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

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM arepl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$;

CREATE FUNCTION arepl.levels_points_after_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM arepl.recalculate_points();
    RETURN NULL;
END;
$$;

CREATE FUNCTION arepl.levels_points_before_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN arepl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') + 1 AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.levels_points_before_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN arepl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') AS INT) + CASE WHEN OLD.status = 'MainList' THEN 0 ELSE 1 END)
        ELSE 0
    END;
    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.max_list_pos() RETURNS integer
    LANGUAGE sql
    AS $$
    SELECT COALESCE(MAX(position), 0) FROM arepl.levels WHERE status = 'MainList';
$$;

CREATE FUNCTION arepl.max_list_pos_legacy() RETURNS integer
    LANGUAGE sql
    AS $$
    SELECT COALESCE(MAX(position), 0) FROM arepl.levels WHERE status IN ('MainList', 'Legacy');
$$;

CREATE FUNCTION arepl.point_formula(pos integer, level_count integer) RETURNS integer
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION arepl.rebuild_position_history_full_view() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    TRUNCATE TABLE arepl.position_history_full_view;

    INSERT INTO arepl.position_history_full_view (
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
        FROM arepl.position_history
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
    SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff
    FROM filtered;
END;
$$;

CREATE FUNCTION arepl.rebuild_submission_daily_stats() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    TRUNCATE
        arepl.submission_daily_total_stats,
        arepl.submission_daily_reviewer_stats,
        arepl.submission_daily_level_stats;

    INSERT INTO arepl.submission_daily_total_stats (
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
        FROM arepl.submission_history h
        INNER JOIN arepl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
            )::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day
    )
    SELECT day, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;

    INSERT INTO arepl.submission_daily_reviewer_stats (
        day,
        reviewer_id,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    SELECT
        DATE(h.timestamp) AS day,
        h.reviewer_id,
        SUM((h.status = 'Accepted'::submission_status)::int)::bigint AS accepted,
        SUM((h.status = 'Denied'::submission_status)::int)::bigint AS denied,
        SUM((h.status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
        SUM(
            (h.status = 'Accepted'::submission_status)::int
            + (h.status = 'Denied'::submission_status)::int
            + (h.status = 'UnderConsideration'::submission_status)::int
        )::bigint AS reviewed
    FROM arepl.submission_history h
    INNER JOIN arepl.submissions s ON s.id = h.submission_id
    WHERE h.reviewer_id IS NOT NULL
    GROUP BY DATE(h.timestamp), h.reviewer_id
    HAVING SUM((h.status = 'Accepted'::submission_status)::int) <> 0
        OR SUM((h.status = 'Denied'::submission_status)::int) <> 0
        OR SUM((h.status = 'UnderConsideration'::submission_status)::int) <> 0;

    INSERT INTO arepl.submission_daily_level_stats (
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
            h.reviewer_id,
            s.level_id,
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
        FROM arepl.submission_history h
        INNER JOIN arepl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            level_id,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
            )::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day, level_id
    )
    SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;
END;
$$;

CREATE FUNCTION arepl.recalculate_points() RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
    UPDATE arepl.levels
    SET points = CASE
        WHEN status = 'MainList' THEN arepl.point_formula(position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
END;
$$;

CREATE FUNCTION arepl.submission_is_only_claim_toggle(old_submission arepl.submissions, new_submission arepl.submissions) RETURNS boolean
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION arepl.submission_log_history() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF arepl.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;

        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.submission_sync_record() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF NEW.status = 'Accepted' THEN
        INSERT INTO arepl.records AS r (
            level_id,
            submitted_by,
            mobile,
            video_url,
			completion_time,
            submission_id
        )
        VALUES (
            NEW.level_id,
            NEW.submitted_by,
            NEW.mobile,
            NEW.video_url,
			NEW.completion_time,
            NEW.id
        )
        ON CONFLICT (level_id, submitted_by)
        DO UPDATE SET
            mobile = EXCLUDED.mobile,
            video_url = EXCLUDED.video_url,
			completion_time = EXCLUDED.completion_time;
			
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.submission_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
$$;

CREATE FUNCTION arepl.update_submission_daily_stats_from_submission() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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

        IF arepl.submission_is_only_claim_toggle(OLD, NEW) THEN
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
    accepted_diff = CASE
        WHEN NEW.status = 'Accepted'::submission_status
             AND NEW.reviewer_id IS NOT NULL
        THEN 1
        ELSE 0
    END;
    denied_diff = (NEW.status = 'Denied'::submission_status)::int;
    under_consideration_diff = (NEW.status = 'UnderConsideration'::submission_status)::int;

    PERFORM arepl.apply_submission_daily_stats_diff(
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
$$;

CREATE FUNCTION arepl.validate_position_insert() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
        highestPos := arepl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := arepl.max_list_pos_legacy() + 1;
        lowestPos := arepl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

CREATE FUNCTION arepl.validate_position_update() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
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
            highestPos := arepl.max_list_pos();
        ELSE
            highestPos := arepl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := arepl.max_list_pos();
        ELSE
            lowestPos := arepl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := arepl.max_list_pos_legacy();
        ELSE
            highestPos := arepl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos
            USING ERRCODE = 'check_violation';
    END IF;

    RETURN NEW;
END;
$$;

CREATE VIEW aredl.badge_level_statistics AS
SELECT r.submitted_by,
       l.id,
       l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS POSITION,
       l.position AS current_position,
       l.level_id,
       l.two_player,
       l.publisher_id,
       l.edel_enjoyment,
       l.nlw_tier,
       l.tags,
       r.is_verification,
       r.achieved_at,
       COALESCE((first_record.submitted_by = r.submitted_by), FALSE) AS is_first_victor,
       FALSE AS is_fastest_time
FROM ((((aredl.records r
         JOIN aredl.levels l ON ((l.id = r.level_id)))
        LEFT JOIN LATERAL
          (SELECT ph.position,
                  ph.status,
                  ph.action_at,
                  ph.ord
           FROM aredl.position_history_full_view ph
           WHERE ((ph.affected_level = r.level_id)
                  AND (ph.action_at <= r.achieved_at))
           ORDER BY ph.action_at DESC, ph.ord DESC
           LIMIT 1) completed_state ON (TRUE))
       LEFT JOIN LATERAL
         (SELECT ph.position,
                 ph.status,
                 ph.action_at,
                 ph.ord
          FROM aredl.position_history_full_view ph
          WHERE ((ph.affected_level = r.level_id)
                 AND (ph.position IS NOT NULL)
                 AND (completed_state.position IS NULL)
                 AND ((completed_state.status IS NULL)
                      OR ((completed_state.status = 'Pending'::public.level_status)
                          AND (ph.action_at >= r.achieved_at))))
          ORDER BY ph.action_at,
                   ph.ord
          LIMIT 1) first_placed_state ON (TRUE))
      LEFT JOIN LATERAL
        (SELECT fr.submitted_by
         FROM aredl.records fr
         WHERE ((fr.level_id = r.level_id)
                AND (fr.is_verification = FALSE))
         ORDER BY fr.achieved_at,
                  fr.created_at,
                  fr.id
         LIMIT 1) first_record ON (TRUE))
WHERE ((completed_state.status = ANY (ARRAY['MainList'::public.level_status,
                                            'Pending'::public.level_status]))
       OR ((completed_state.status IS NULL)
           AND (first_placed_state.position IS NOT NULL)));

CREATE VIEW aredl.clan_member_points AS WITH clan_records AS
  (SELECT cm.clan_id,
          r.submitted_by,
          l.points,
          count(*) OVER (PARTITION BY cm.clan_id,
                                      r.level_id) AS completion_count
   FROM (((aredl.records r
           JOIN public.clan_members cm ON ((cm.user_id = r.submitted_by)))
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN aredl.levels l ON ((l.id = r.level_id)))
   WHERE (l.status <> 'Removed'::public.level_status) )
SELECT clan_records.clan_id,
       clan_records.submitted_by,
       count(*) AS completed_levels,
       sum(((clan_records.points)::double precision / (clan_records.completion_count)::double precision)) AS contributed_points
FROM clan_records
GROUP BY clan_records.clan_id,
         clan_records.submitted_by;

CREATE MATERIALIZED VIEW aredl.clans_created_levels AS WITH explicit_creators AS
  (SELECT cm.clan_id,
          l.id AS level_id,
          lc.user_id AS creator_id,
          l.position AS order_pos
   FROM ((aredl.levels_created lc
          JOIN aredl.levels l ON ((l.id = lc.level_id)))
         JOIN public.clan_members cm ON ((cm.user_id = lc.user_id)))
   WHERE (l.status <> 'Removed'::public.level_status) ),
                                                            levels_without_explicit_creators AS
  (SELECT cm.clan_id,
          l.id AS level_id,
          l.publisher_id AS creator_id,
          l.position AS order_pos
   FROM ((aredl.levels l
          JOIN public.clan_members cm ON ((cm.user_id = l.publisher_id)))
         LEFT JOIN aredl.levels_created lc ON ((lc.level_id = l.id)))
   WHERE ((lc.level_id IS NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT explicit_creators.clan_id,
       explicit_creators.level_id,
       explicit_creators.creator_id,
       explicit_creators.order_pos
FROM explicit_creators
UNION
SELECT levels_without_explicit_creators.clan_id,
       levels_without_explicit_creators.level_id,
       levels_without_explicit_creators.creator_id,
       levels_without_explicit_creators.order_pos
FROM levels_without_explicit_creators WITH DATA;

CREATE MATERIALIZED VIEW aredl.clans_leaderboard AS WITH completed_levels AS
  (SELECT DISTINCT cm.clan_id,
                   r.level_id
   FROM (((aredl.records r
           JOIN public.clan_members cm ON ((r.submitted_by = cm.user_id)))
          JOIN public.users u ON ((r.submitted_by = u.id)))
         JOIN aredl.levels l ON ((r.level_id = l.id)))
   WHERE ((u.ban_level <= 1)
          AND (l.status = ANY (ARRAY['MainList'::public.level_status,
                                     'Pending'::public.level_status]))) ),
                                                         level_points AS
  (SELECT c.clan_id,
          (COALESCE(sum(l.points), (0)::bigint))::integer AS level_points
   FROM (completed_levels c
         JOIN aredl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.clan_id),
                                                         hardest_position AS
  (SELECT c.clan_id,
          min(l.position) AS POSITION
   FROM (completed_levels c
         JOIN aredl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.clan_id),
                                                         hardest AS
  (SELECT hp.clan_id,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN aredl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                         level_count AS
  (SELECT completed_levels.clan_id,
          count(*) AS c
   FROM completed_levels
   GROUP BY completed_levels.clan_id),
                                                         user_count AS
  (SELECT clan_members.clan_id,
          count(*) AS c
   FROM public.clan_members
   GROUP BY clan_members.clan_id)
SELECT (rank() OVER (
                     ORDER BY lp.level_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       lp.clan_id,
       lp.level_points,
       (COALESCE(uc.c, (0)::bigint))::integer AS members_count,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes
FROM (((level_points lp
        LEFT JOIN hardest h ON ((h.clan_id = lp.clan_id)))
       LEFT JOIN level_count lc ON ((lc.clan_id = lp.clan_id)))
      LEFT JOIN user_count uc ON ((uc.clan_id = lp.clan_id))) WITH DATA;

CREATE VIEW aredl.completed_packs AS WITH pcl AS
  (SELECT pl_1.pack_id,
          count(*) AS lc
   FROM aredl.pack_levels pl_1
   GROUP BY pl_1.pack_id)
SELECT r.submitted_by AS user_id,
       pl.pack_id,
       max(r.achieved_at) AS completed_at
FROM (((aredl.records r
        JOIN public.users u ON (((u.id = r.submitted_by)
                                 AND (u.ban_level <= 2))))
       JOIN aredl.pack_levels pl ON ((pl.level_id = r.level_id)))
      JOIN pcl ON ((pcl.pack_id = pl.pack_id)))
GROUP BY r.submitted_by,
         pl.pack_id,
         pcl.lc
HAVING (count(*) = pcl.lc);

CREATE MATERIALIZED VIEW aredl.country_created_levels AS WITH explicit_creators AS
  (SELECT u.country,
          l.id AS level_id,
          lc.user_id AS creator_id,
          l.position AS order_pos
   FROM ((aredl.levels_created lc
          JOIN aredl.levels l ON ((l.id = lc.level_id)))
         JOIN public.users u ON ((u.id = lc.user_id)))
   WHERE ((u.country IS NOT NULL)
          AND (l.status <> 'Removed'::public.level_status)) ),
                                                              levels_without_explicit_creators AS
  (SELECT u.country,
          l.id AS level_id,
          l.publisher_id AS creator_id,
          l.position AS order_pos
   FROM ((aredl.levels l
          JOIN public.users u ON ((u.id = l.publisher_id)))
         LEFT JOIN aredl.levels_created lc ON ((lc.level_id = l.id)))
   WHERE ((u.country IS NOT NULL)
          AND (lc.level_id IS NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT explicit_creators.country,
       explicit_creators.level_id,
       explicit_creators.creator_id,
       explicit_creators.order_pos
FROM explicit_creators
UNION
SELECT levels_without_explicit_creators.country,
       levels_without_explicit_creators.level_id,
       levels_without_explicit_creators.creator_id,
       levels_without_explicit_creators.order_pos
FROM levels_without_explicit_creators WITH DATA;

CREATE MATERIALIZED VIEW aredl.country_leaderboard AS WITH completed_levels AS
  (SELECT DISTINCT u.country,
                   r.level_id
   FROM ((aredl.records r
          JOIN public.users u ON ((r.submitted_by = u.id)))
         JOIN aredl.levels l ON ((r.level_id = l.id)))
   WHERE ((u.ban_level <= 1)
          AND (u.country IS NOT NULL)
          AND (u.country <> 0)
          AND (l.status = ANY (ARRAY['MainList'::public.level_status,
                                     'Pending'::public.level_status]))) ),
                                                           level_points AS
  (SELECT c.country,
          (COALESCE(sum(l.points), (0)::bigint))::integer AS level_points
   FROM (completed_levels c
         JOIN aredl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.country),
                                                           hardest_position AS
  (SELECT c.country,
          min(l.position) AS POSITION
   FROM (completed_levels c
         JOIN aredl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.country),
                                                           hardest AS
  (SELECT hp.country,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN aredl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                           level_count AS
  (SELECT completed_levels.country,
          count(*) AS c
   FROM completed_levels
   GROUP BY completed_levels.country),
                                                           user_count AS
  (SELECT users.country,
          count(*) AS c
   FROM public.users
   WHERE ((users.ban_level <= 1)
          AND (users.country IS NOT NULL)
          AND (users.country <> 0))
   GROUP BY users.country)
SELECT (rank() OVER (
                     ORDER BY lp.level_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       lp.country,
       lp.level_points,
       (COALESCE(uc.c, (0)::bigint))::integer AS members_count,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes
FROM (((level_points lp
        LEFT JOIN hardest h ON ((h.country = lp.country)))
       LEFT JOIN level_count lc ON ((lc.country = lp.country)))
      LEFT JOIN user_count uc ON ((uc.country = lp.country))) WITH DATA;

CREATE VIEW aredl.min_placement_clans_records AS WITH subquery AS
  (SELECT r.id,
          r.level_id,
          r.submitted_by,
          r.mobile,
          r.video_url,
          r.created_at,
          r.updated_at,
          r.is_verification,
          r.hide_video,
          r.submission_id,
          r.achieved_at,
          cm.clan_id,
          row_number() OVER (PARTITION BY r.level_id,
                                          cm.clan_id
                             ORDER BY r.achieved_at) AS order_pos,
                            count(*) OVER (PARTITION BY r.level_id,
                                                        cm.clan_id) AS completion_count
   FROM (((aredl.records r
           JOIN public.clan_members cm ON ((cm.user_id = r.submitted_by)))
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN aredl.levels l ON ((l.id = r.level_id)))
   WHERE (l.status <> 'Removed'::public.level_status) )
SELECT subquery.id,
       subquery.level_id,
       subquery.submitted_by,
       subquery.mobile,
       subquery.video_url,
       subquery.created_at,
       subquery.updated_at,
       subquery.is_verification,
       subquery.hide_video,
       subquery.submission_id,
       subquery.achieved_at,
       subquery.clan_id,
       subquery.order_pos,
       subquery.completion_count
FROM subquery
WHERE (subquery.order_pos = 1);

CREATE VIEW aredl.min_placement_country_records AS WITH subquery AS
  (SELECT r.id,
          r.level_id,
          r.submitted_by,
          r.mobile,
          r.video_url,
          r.created_at,
          r.updated_at,
          r.is_verification,
          r.hide_video,
          r.submission_id,
          r.achieved_at,
          u.country,
          row_number() OVER (PARTITION BY r.level_id,
                                          u.country
                             ORDER BY r.achieved_at) AS order_pos,
                            count(*) OVER (PARTITION BY r.level_id,
                                                        u.country) AS completion_count
   FROM ((aredl.records r
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN aredl.levels l ON ((l.id = r.level_id)))
   WHERE ((u.country IS NOT NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT subquery.id,
       subquery.level_id,
       subquery.submitted_by,
       subquery.mobile,
       subquery.video_url,
       subquery.created_at,
       subquery.updated_at,
       subquery.is_verification,
       subquery.hide_video,
       subquery.submission_id,
       subquery.achieved_at,
       subquery.country,
       subquery.order_pos,
       subquery.completion_count
FROM subquery
WHERE (subquery.order_pos = 1);

CREATE VIEW aredl.packs_points AS
SELECT p.id,
       p.name,
       p.tier,
       (round(((sum(l.points))::numeric * 0.5)))::integer AS points
FROM ((aredl.packs p
       JOIN aredl.pack_levels pl ON ((p.id = pl.pack_id)))
      JOIN aredl.levels l ON ((l.id = pl.level_id)))
GROUP BY p.id;

CREATE MATERIALIZED VIEW aredl.record_totals AS
SELECT NULL::UUID AS level_id,
       count(*) FILTER (
                        WHERE (r.is_verification = FALSE)) AS records,
       count(*) FILTER (
                        WHERE (r.is_verification = TRUE)) AS verifications
FROM (aredl.records r
      JOIN public.users u ON (((u.id = r.submitted_by)
                               AND (u.ban_level <= 2))))
UNION ALL
SELECT r.level_id,
       count(*) FILTER (
                        WHERE (r.is_verification = FALSE)) AS records,
       count(*) FILTER (
                        WHERE (r.is_verification = TRUE)) AS verifications
FROM (aredl.records r
      JOIN public.users u ON (((u.id = r.submitted_by)
                               AND (u.ban_level <= 2))))
GROUP BY r.level_id WITH DATA;

CREATE MATERIALIZED VIEW aredl.submission_totals AS
SELECT NULL::UUID AS level_id,
       count(*) AS submissions,
       (100.00)::double precision AS percent_of_queue
FROM aredl.submissions
WHERE (submissions.status = 'Pending'::public.submission_status)
UNION ALL
SELECT submissions.level_id,
       count(*) AS submissions,
       (round((((count(*))::numeric * 100.0) / sum(count(*)) OVER ()), 2))::double precision AS percent_of_queue
FROM aredl.submissions
WHERE (submissions.status = 'Pending'::public.submission_status)
GROUP BY submissions.level_id
ORDER BY 2 DESC WITH DATA;

CREATE VIEW aredl.user_pack_points AS
SELECT cp.user_id,
       (sum(p.points))::integer AS points
FROM (aredl.completed_packs cp
      JOIN aredl.packs_points p ON ((p.id = cp.pack_id)))
GROUP BY cp.user_id;

CREATE MATERIALIZED VIEW aredl.user_leaderboard AS WITH user_points AS
  (SELECT u.id AS user_id,
          u.country,
          ((COALESCE(sum(l.points), (0)::bigint) + COALESCE(pp.points, 0)))::integer AS total_points,
          COALESCE(pp.points, 0) AS pack_points
   FROM (((public.users u
           LEFT JOIN aredl.records r ON ((u.id = r.submitted_by)))
          LEFT JOIN aredl.levels l ON (((r.level_id = l.id)
                                        AND (l.status = 'MainList'::public.level_status))))
         LEFT JOIN aredl.user_pack_points pp ON ((pp.user_id = r.submitted_by)))
   WHERE (u.ban_level = 0)
   GROUP BY u.id,
            u.country,
            pp.points),
                                                        hardest_position AS
  (SELECT r.submitted_by AS user_id,
          min(l.position) AS POSITION
   FROM (aredl.records r
         JOIN aredl.levels l ON ((r.level_id = l.id)))
   WHERE (l.status = 'MainList'::public.level_status)
   GROUP BY r.submitted_by),
                                                        hardest AS
  (SELECT hp.user_id,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN aredl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                        level_count AS
  (SELECT r.submitted_by AS id,
          count(*) AS c
   FROM (aredl.records r
         JOIN aredl.levels l ON ((r.level_id = l.id)))
   WHERE (l.status = ANY (ARRAY['MainList'::public.level_status,
                                'Pending'::public.level_status]))
   GROUP BY r.submitted_by)
SELECT (rank() OVER (
                     ORDER BY up.total_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY (up.total_points - up.pack_points) DESC))::integer AS raw_rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY up.total_points DESC))::integer AS country_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY (up.total_points - up.pack_points) DESC))::integer AS country_raw_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS country_extremes_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY h.position))::integer AS country_hardest_rank,
       up.user_id,
       up.country,
       up.total_points,
       up.pack_points,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes,
       cm.clan_id
FROM (((user_points up
        LEFT JOIN hardest h ON ((h.user_id = up.user_id)))
       LEFT JOIN level_count lc ON ((lc.id = up.user_id)))
      LEFT JOIN public.clan_members cm ON ((cm.user_id = up.user_id))) WITH DATA;

CREATE VIEW arepl.badge_level_statistics AS
SELECT r.submitted_by,
       l.id,
       l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS POSITION,
       l.position AS current_position,
       l.level_id,
       l.two_player,
       l.publisher_id,
       l.edel_enjoyment,
       l.nlw_tier,
       l.tags,
       r.is_verification,
       r.achieved_at,
       COALESCE((first_record.submitted_by = r.submitted_by), FALSE) AS is_first_victor,
       COALESCE((fastest_record.submitted_by = r.submitted_by), FALSE) AS is_fastest_time
FROM (((((arepl.records r
          JOIN arepl.levels l ON ((l.id = r.level_id)))
         LEFT JOIN LATERAL
           (SELECT ph.position,
                   ph.status,
                   ph.action_at,
                   ph.ord
            FROM arepl.position_history_full_view ph
            WHERE ((ph.affected_level = r.level_id)
                   AND (ph.action_at <= r.achieved_at))
            ORDER BY ph.action_at DESC, ph.ord DESC
            LIMIT 1) completed_state ON (TRUE))
        LEFT JOIN LATERAL
          (SELECT ph.position,
                  ph.status,
                  ph.action_at,
                  ph.ord
           FROM arepl.position_history_full_view ph
           WHERE ((ph.affected_level = r.level_id)
                  AND (ph.position IS NOT NULL)
                  AND (completed_state.position IS NULL)
                  AND ((completed_state.status IS NULL)
                       OR ((completed_state.status = 'Pending'::public.level_status)
                           AND (ph.action_at >= r.achieved_at))))
           ORDER BY ph.action_at,
                    ph.ord
           LIMIT 1) first_placed_state ON (TRUE))
       LEFT JOIN LATERAL
         (SELECT fr.submitted_by
          FROM arepl.records fr
          WHERE ((fr.level_id = r.level_id)
                 AND (fr.is_verification = FALSE))
          ORDER BY fr.achieved_at,
                   fr.created_at,
                   fr.id
          LIMIT 1) first_record ON (TRUE))
      LEFT JOIN LATERAL
        (SELECT fr.submitted_by
         FROM arepl.records fr
         WHERE ((fr.level_id = r.level_id)
                AND (fr.is_verification = FALSE))
         ORDER BY fr.completion_time,
                  fr.achieved_at,
                  fr.id
         LIMIT 1) fastest_record ON (TRUE))
WHERE ((completed_state.status = ANY (ARRAY['MainList'::public.level_status,
                                            'Pending'::public.level_status]))
       OR ((completed_state.status IS NULL)
           AND (first_placed_state.position IS NOT NULL)));

CREATE VIEW arepl.clan_member_points AS WITH clan_records AS
  (SELECT cm.clan_id,
          r.submitted_by,
          l.points,
          count(*) OVER (PARTITION BY cm.clan_id,
                                      r.level_id) AS completion_count
   FROM (((arepl.records r
           JOIN public.clan_members cm ON ((cm.user_id = r.submitted_by)))
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN arepl.levels l ON ((l.id = r.level_id)))
   WHERE (l.status <> 'Removed'::public.level_status) )
SELECT clan_records.clan_id,
       clan_records.submitted_by,
       count(*) AS completed_levels,
       sum(((clan_records.points)::double precision / (clan_records.completion_count)::double precision)) AS contributed_points
FROM clan_records
GROUP BY clan_records.clan_id,
         clan_records.submitted_by;

CREATE MATERIALIZED VIEW arepl.clans_created_levels AS WITH explicit_creators AS
  (SELECT cm.clan_id,
          l.id AS level_id,
          lc.user_id AS creator_id,
          l.position AS order_pos
   FROM ((arepl.levels_created lc
          JOIN arepl.levels l ON ((l.id = lc.level_id)))
         JOIN public.clan_members cm ON ((cm.user_id = lc.user_id)))
   WHERE (l.status <> 'Removed'::public.level_status) ),
                                                            levels_without_explicit_creators AS
  (SELECT cm.clan_id,
          l.id AS level_id,
          l.publisher_id AS creator_id,
          l.position AS order_pos
   FROM ((arepl.levels l
          JOIN public.clan_members cm ON ((cm.user_id = l.publisher_id)))
         LEFT JOIN arepl.levels_created lc ON ((lc.level_id = l.id)))
   WHERE ((lc.level_id IS NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT explicit_creators.clan_id,
       explicit_creators.level_id,
       explicit_creators.creator_id,
       explicit_creators.order_pos
FROM explicit_creators
UNION
SELECT levels_without_explicit_creators.clan_id,
       levels_without_explicit_creators.level_id,
       levels_without_explicit_creators.creator_id,
       levels_without_explicit_creators.order_pos
FROM levels_without_explicit_creators WITH DATA;

CREATE MATERIALIZED VIEW arepl.clans_leaderboard AS WITH completed_levels AS
  (SELECT DISTINCT cm.clan_id,
                   r.level_id
   FROM (((arepl.records r
           JOIN public.clan_members cm ON ((r.submitted_by = cm.user_id)))
          JOIN public.users u ON ((r.submitted_by = u.id)))
         JOIN arepl.levels l ON ((r.level_id = l.id)))
   WHERE ((u.ban_level <= 1)
          AND (l.status = ANY (ARRAY['MainList'::public.level_status,
                                     'Pending'::public.level_status]))) ),
                                                         level_points AS
  (SELECT c.clan_id,
          (COALESCE(sum(l.points), (0)::bigint))::integer AS level_points
   FROM (completed_levels c
         JOIN arepl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.clan_id),
                                                         hardest_position AS
  (SELECT c.clan_id,
          min(l.position) AS POSITION
   FROM (completed_levels c
         JOIN arepl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.clan_id),
                                                         hardest AS
  (SELECT hp.clan_id,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN arepl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                         level_count AS
  (SELECT completed_levels.clan_id,
          count(*) AS c
   FROM completed_levels
   GROUP BY completed_levels.clan_id),
                                                         user_count AS
  (SELECT clan_members.clan_id,
          count(*) AS c
   FROM public.clan_members
   GROUP BY clan_members.clan_id)
SELECT (rank() OVER (
                     ORDER BY lp.level_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       lp.clan_id,
       lp.level_points,
       (COALESCE(uc.c, (0)::bigint))::integer AS members_count,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes
FROM (((level_points lp
        LEFT JOIN hardest h ON ((h.clan_id = lp.clan_id)))
       LEFT JOIN level_count lc ON ((lc.clan_id = lp.clan_id)))
      LEFT JOIN user_count uc ON ((uc.clan_id = lp.clan_id))) WITH DATA;

CREATE VIEW arepl.completed_packs AS WITH pcl AS
  (SELECT pl_1.pack_id,
          count(*) AS lc
   FROM arepl.pack_levels pl_1
   GROUP BY pl_1.pack_id)
SELECT r.submitted_by AS user_id,
       pl.pack_id,
       max(r.achieved_at) AS completed_at
FROM (((arepl.records r
        JOIN public.users u ON (((u.id = r.submitted_by)
                                 AND (u.ban_level <= 2))))
       JOIN arepl.pack_levels pl ON ((pl.level_id = r.level_id)))
      JOIN pcl ON ((pcl.pack_id = pl.pack_id)))
GROUP BY r.submitted_by,
         pl.pack_id,
         pcl.lc
HAVING (count(*) = pcl.lc);

CREATE MATERIALIZED VIEW arepl.country_created_levels AS WITH explicit_creators AS
  (SELECT u.country,
          l.id AS level_id,
          lc.user_id AS creator_id,
          l.position AS order_pos
   FROM ((arepl.levels_created lc
          JOIN arepl.levels l ON ((l.id = lc.level_id)))
         JOIN public.users u ON ((u.id = lc.user_id)))
   WHERE ((u.country IS NOT NULL)
          AND (l.status <> 'Removed'::public.level_status)) ),
                                                              levels_without_explicit_creators AS
  (SELECT u.country,
          l.id AS level_id,
          l.publisher_id AS creator_id,
          l.position AS order_pos
   FROM ((arepl.levels l
          JOIN public.users u ON ((u.id = l.publisher_id)))
         LEFT JOIN arepl.levels_created lc ON ((lc.level_id = l.id)))
   WHERE ((u.country IS NOT NULL)
          AND (lc.level_id IS NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT explicit_creators.country,
       explicit_creators.level_id,
       explicit_creators.creator_id,
       explicit_creators.order_pos
FROM explicit_creators
UNION
SELECT levels_without_explicit_creators.country,
       levels_without_explicit_creators.level_id,
       levels_without_explicit_creators.creator_id,
       levels_without_explicit_creators.order_pos
FROM levels_without_explicit_creators WITH DATA;

CREATE MATERIALIZED VIEW arepl.country_leaderboard AS WITH completed_levels AS
  (SELECT DISTINCT u.country,
                   r.level_id
   FROM ((arepl.records r
          JOIN public.users u ON ((r.submitted_by = u.id)))
         JOIN arepl.levels l ON ((r.level_id = l.id)))
   WHERE ((u.ban_level <= 1)
          AND (u.country IS NOT NULL)
          AND (u.country <> 0)
          AND (l.status = ANY (ARRAY['MainList'::public.level_status,
                                     'Pending'::public.level_status]))) ),
                                                           level_points AS
  (SELECT c.country,
          (COALESCE(sum(l.points), (0)::bigint))::integer AS level_points
   FROM (completed_levels c
         JOIN arepl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.country),
                                                           hardest_position AS
  (SELECT c.country,
          min(l.position) AS POSITION
   FROM (completed_levels c
         JOIN arepl.levels l ON ((c.level_id = l.id)))
   GROUP BY c.country),
                                                           hardest AS
  (SELECT hp.country,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN arepl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                           level_count AS
  (SELECT completed_levels.country,
          count(*) AS c
   FROM completed_levels
   GROUP BY completed_levels.country),
                                                           user_count AS
  (SELECT users.country,
          count(*) AS c
   FROM public.users
   WHERE ((users.ban_level <= 1)
          AND (users.country IS NOT NULL)
          AND (users.country <> 0))
   GROUP BY users.country)
SELECT (rank() OVER (
                     ORDER BY lp.level_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       lp.country,
       lp.level_points,
       (COALESCE(uc.c, (0)::bigint))::integer AS members_count,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes
FROM (((level_points lp
        LEFT JOIN hardest h ON ((h.country = lp.country)))
       LEFT JOIN level_count lc ON ((lc.country = lp.country)))
      LEFT JOIN user_count uc ON ((uc.country = lp.country))) WITH DATA;

CREATE VIEW arepl.min_placement_clans_records AS WITH subquery AS
  (SELECT r.id,
          r.level_id,
          r.submitted_by,
          r.mobile,
          r.video_url,
          r.created_at,
          r.updated_at,
          r.is_verification,
          r.completion_time,
          r.hide_video,
          r.submission_id,
          r.achieved_at,
          cm.clan_id,
          row_number() OVER (PARTITION BY r.level_id,
                                          cm.clan_id
                             ORDER BY r.achieved_at) AS order_pos,
                            count(*) OVER (PARTITION BY r.level_id,
                                                        cm.clan_id) AS completion_count
   FROM (((arepl.records r
           JOIN public.clan_members cm ON ((cm.user_id = r.submitted_by)))
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN arepl.levels l ON ((l.id = r.level_id)))
   WHERE (l.status <> 'Removed'::public.level_status) )
SELECT subquery.id,
       subquery.level_id,
       subquery.submitted_by,
       subquery.mobile,
       subquery.video_url,
       subquery.created_at,
       subquery.updated_at,
       subquery.is_verification,
       subquery.completion_time,
       subquery.hide_video,
       subquery.submission_id,
       subquery.achieved_at,
       subquery.clan_id,
       subquery.order_pos,
       subquery.completion_count
FROM subquery
WHERE (subquery.order_pos = 1);

CREATE VIEW arepl.min_placement_country_records AS WITH subquery AS
  (SELECT r.id,
          r.level_id,
          r.submitted_by,
          r.mobile,
          r.video_url,
          r.created_at,
          r.updated_at,
          r.is_verification,
          r.completion_time,
          r.hide_video,
          r.submission_id,
          r.achieved_at,
          u.country,
          row_number() OVER (PARTITION BY r.level_id,
                                          u.country
                             ORDER BY r.achieved_at) AS order_pos,
                            count(*) OVER (PARTITION BY r.level_id,
                                                        u.country) AS completion_count
   FROM ((arepl.records r
          JOIN public.users u ON (((u.id = r.submitted_by)
                                   AND (u.ban_level <= 1))))
         JOIN arepl.levels l ON ((l.id = r.level_id)))
   WHERE ((u.country IS NOT NULL)
          AND (l.status <> 'Removed'::public.level_status)) )
SELECT subquery.id,
       subquery.level_id,
       subquery.submitted_by,
       subquery.mobile,
       subquery.video_url,
       subquery.created_at,
       subquery.updated_at,
       subquery.is_verification,
       subquery.completion_time,
       subquery.hide_video,
       subquery.submission_id,
       subquery.achieved_at,
       subquery.country,
       subquery.order_pos,
       subquery.completion_count
FROM subquery
WHERE (subquery.order_pos = 1);

CREATE VIEW arepl.packs_points AS
SELECT p.id,
       p.name,
       p.tier,
       (round(((sum(l.points))::numeric * 0.5)))::integer AS points
FROM ((arepl.packs p
       JOIN arepl.pack_levels pl ON ((p.id = pl.pack_id)))
      JOIN arepl.levels l ON ((l.id = pl.level_id)))
GROUP BY p.id;

CREATE MATERIALIZED VIEW arepl.record_totals AS
SELECT NULL::UUID AS level_id,
       count(*) FILTER (
                        WHERE (r.is_verification = FALSE)) AS records,
       count(*) FILTER (
                        WHERE (r.is_verification = TRUE)) AS verifications
FROM (arepl.records r
      JOIN public.users u ON (((u.id = r.submitted_by)
                               AND (u.ban_level <= 2))))
UNION ALL
SELECT r.level_id,
       count(*) FILTER (
                        WHERE (r.is_verification = FALSE)) AS records,
       count(*) FILTER (
                        WHERE (r.is_verification = TRUE)) AS verifications
FROM (arepl.records r
      JOIN public.users u ON (((u.id = r.submitted_by)
                               AND (u.ban_level <= 2))))
GROUP BY r.level_id WITH DATA;

CREATE MATERIALIZED VIEW arepl.submission_totals AS
SELECT NULL::UUID AS level_id,
       count(*) AS submissions,
       (100.00)::double precision AS percent_of_queue
FROM arepl.submissions
WHERE (submissions.status = 'Pending'::public.submission_status)
UNION ALL
SELECT submissions.level_id,
       count(*) AS submissions,
       (round((((count(*))::numeric * 100.0) / sum(count(*)) OVER ()), 2))::double precision AS percent_of_queue
FROM arepl.submissions
WHERE (submissions.status = 'Pending'::public.submission_status)
GROUP BY submissions.level_id
ORDER BY 2 DESC WITH DATA;

CREATE VIEW arepl.user_pack_points AS
SELECT cp.user_id,
       (sum(p.points))::integer AS points
FROM (arepl.completed_packs cp
      JOIN arepl.packs_points p ON ((p.id = cp.pack_id)))
GROUP BY cp.user_id;

CREATE MATERIALIZED VIEW arepl.user_leaderboard AS WITH user_points AS
  (SELECT u.id AS user_id,
          u.country,
          ((COALESCE(sum(l.points), (0)::bigint) + COALESCE(pp.points, 0)))::integer AS total_points,
          COALESCE(pp.points, 0) AS pack_points
   FROM (((public.users u
           LEFT JOIN arepl.records r ON ((u.id = r.submitted_by)))
          LEFT JOIN arepl.levels l ON (((r.level_id = l.id)
                                        AND (l.status = 'MainList'::public.level_status))))
         LEFT JOIN arepl.user_pack_points pp ON ((pp.user_id = r.submitted_by)))
   WHERE (u.ban_level = 0)
   GROUP BY u.id,
            u.country,
            pp.points),
                                                        hardest_position AS
  (SELECT r.submitted_by AS user_id,
          min(l.position) AS POSITION
   FROM (arepl.records r
         JOIN arepl.levels l ON ((r.level_id = l.id)))
   WHERE (l.status = 'MainList'::public.level_status)
   GROUP BY r.submitted_by),
                                                        hardest AS
  (SELECT hp.user_id,
          hp.position,
          l.id AS level_id
   FROM (hardest_position hp
         JOIN arepl.levels l ON (((hp.position = l.position)
                                  AND (l.status = 'MainList'::public.level_status))))),
                                                        level_count AS
  (SELECT r.submitted_by AS id,
          count(*) AS c
   FROM (arepl.records r
         JOIN arepl.levels l ON ((r.level_id = l.id)))
   WHERE (l.status = ANY (ARRAY['MainList'::public.level_status,
                                'Pending'::public.level_status]))
   GROUP BY r.submitted_by)
SELECT (rank() OVER (
                     ORDER BY up.total_points DESC))::integer AS rank,
       (rank() OVER (
                     ORDER BY (up.total_points - up.pack_points) DESC))::integer AS raw_rank,
       (rank() OVER (
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS extremes_rank,
       (rank() OVER (
                     ORDER BY h.position))::integer AS hardest_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY up.total_points DESC))::integer AS country_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY (up.total_points - up.pack_points) DESC))::integer AS country_raw_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY COALESCE(lc.c, (0)::bigint) DESC))::integer AS country_extremes_rank,
       (rank() OVER (PARTITION BY up.country
                     ORDER BY h.position))::integer AS country_hardest_rank,
       up.user_id,
       up.country,
       up.total_points,
       up.pack_points,
       h.level_id AS hardest,
       (COALESCE(lc.c, (0)::bigint))::integer AS extremes,
       cm.clan_id
FROM (((user_points up
        LEFT JOIN hardest h ON ((h.user_id = up.user_id)))
       LEFT JOIN level_count lc ON ((lc.id = up.user_id)))
      LEFT JOIN public.clan_members cm ON ((cm.user_id = up.user_id))) WITH DATA;

CREATE INDEX aredl_clans_created_levels_clan_idx ON aredl.clans_created_levels (clan_id, order_pos, level_id, creator_id);
CREATE INDEX aredl_country_created_levels_country_idx ON aredl.country_created_levels (country, order_pos, level_id, creator_id);
CREATE UNIQUE INDEX aredl_record_totals_idx ON aredl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));
CREATE INDEX aredl_user_leaderboard_rank_user_idx ON aredl.user_leaderboard (rank, user_id);
CREATE UNIQUE INDEX aredl_user_leaderboard_user_id_idx ON aredl.user_leaderboard (user_id);
CREATE INDEX arepl_clans_created_levels_clan_idx ON arepl.clans_created_levels (clan_id, order_pos, level_id, creator_id);
CREATE INDEX arepl_country_created_levels_country_idx ON arepl.country_created_levels (country, order_pos, level_id, creator_id);
CREATE UNIQUE INDEX arepl_record_totals_idx ON arepl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));
CREATE INDEX arepl_user_leaderboard_rank_user_idx ON arepl.user_leaderboard (rank, user_id);
CREATE UNIQUE INDEX arepl_user_leaderboard_user_id_idx ON arepl.user_leaderboard (user_id);

CREATE TRIGGER level_move AFTER UPDATE OF position, status ON aredl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION aredl.level_move();
CREATE TRIGGER level_place AFTER INSERT ON aredl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION aredl.level_place();
CREATE TRIGGER level_place_history AFTER INSERT ON aredl.levels FOR EACH ROW EXECUTE FUNCTION aredl.level_place_history();
CREATE TRIGGER levels_points_after_insert AFTER INSERT ON aredl.levels FOR EACH STATEMENT EXECUTE FUNCTION aredl.levels_points_after_insert();
CREATE TRIGGER levels_points_before_insert BEFORE INSERT ON aredl.levels FOR EACH ROW EXECUTE FUNCTION aredl.levels_points_before_insert();
CREATE TRIGGER levels_points_before_update BEFORE UPDATE OF position, status ON aredl.levels FOR EACH ROW EXECUTE FUNCTION aredl.levels_points_before_update();
CREATE TRIGGER submission_daily_stats_ins AFTER INSERT ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_daily_stats_upd AFTER UPDATE ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_log_history_ins AFTER INSERT ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_log_history();
CREATE TRIGGER submission_log_history_upd AFTER UPDATE ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_log_history();
CREATE TRIGGER submission_sync_record_ins AFTER INSERT ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_sync_record();
CREATE TRIGGER submission_sync_record_upd AFTER UPDATE OF status, mobile, video_url ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_sync_record();
CREATE TRIGGER submission_updated_at BEFORE UPDATE ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_updated_at();
CREATE TRIGGER validate_position_insert BEFORE INSERT ON aredl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION aredl.validate_position_insert();
CREATE TRIGGER validate_position_update BEFORE UPDATE OF position, status ON aredl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION aredl.validate_position_update();
CREATE TRIGGER level_move AFTER UPDATE OF position, status ON arepl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION arepl.level_move();
CREATE TRIGGER level_place AFTER INSERT ON arepl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION arepl.level_place();
CREATE TRIGGER level_place_history AFTER INSERT ON arepl.levels FOR EACH ROW EXECUTE FUNCTION arepl.level_place_history();
CREATE TRIGGER levels_points_after_insert AFTER INSERT ON arepl.levels FOR EACH STATEMENT EXECUTE FUNCTION arepl.levels_points_after_insert();
CREATE TRIGGER levels_points_before_insert BEFORE INSERT ON arepl.levels FOR EACH ROW EXECUTE FUNCTION arepl.levels_points_before_insert();
CREATE TRIGGER levels_points_before_update BEFORE UPDATE OF position, status ON arepl.levels FOR EACH ROW EXECUTE FUNCTION arepl.levels_points_before_update();
CREATE TRIGGER submission_daily_stats_ins AFTER INSERT ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_daily_stats_upd AFTER UPDATE ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.update_submission_daily_stats_from_submission();
CREATE TRIGGER submission_log_history_ins AFTER INSERT ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_log_history();
CREATE TRIGGER submission_log_history_upd AFTER UPDATE ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_log_history();
CREATE TRIGGER submission_sync_record_ins AFTER INSERT ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_sync_record();
CREATE TRIGGER submission_sync_record_upd AFTER UPDATE OF status, mobile, video_url, completion_time ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_sync_record();
CREATE TRIGGER submission_updated_at BEFORE UPDATE ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_updated_at();
CREATE TRIGGER validate_position_insert BEFORE INSERT ON arepl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION arepl.validate_position_insert();

CREATE TRIGGER validate_position_update BEFORE UPDATE OF position, status ON arepl.levels FOR EACH ROW WHEN ((pg_trigger_depth() < 1)) EXECUTE FUNCTION arepl.validate_position_update();

CREATE OR REPLACE FUNCTION public.merge_users(p_primary_user uuid, p_secondary_user uuid)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
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
        FROM aredl.submissions primary_s
        JOIN aredl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE aredl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM aredl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM aredl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- same for arepl
    WITH pairs AS (
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
        FROM arepl.submissions primary_s
        JOIN arepl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE arepl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM arepl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM arepl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- recalculate stats

    INSERT INTO aredl.submission_daily_reviewer_stats AS stats
        (day, reviewer_id, accepted, denied, under_consideration, reviewed)
    SELECT day, p_primary_user, accepted, denied, under_consideration, reviewed
    FROM aredl.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user
    ON CONFLICT (day, reviewer_id) DO UPDATE
    SET accepted = stats.accepted + EXCLUDED.accepted,
        denied = stats.denied + EXCLUDED.denied,
        under_consideration = stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = stats.reviewed + EXCLUDED.reviewed;

    DELETE FROM aredl.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user;

    INSERT INTO arepl.submission_daily_reviewer_stats AS stats
        (day, reviewer_id, accepted, denied, under_consideration, reviewed)
    SELECT day, p_primary_user, accepted, denied, under_consideration, reviewed
    FROM arepl.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user
    ON CONFLICT (day, reviewer_id) DO UPDATE
    SET accepted = stats.accepted + EXCLUDED.accepted,
        denied = stats.denied + EXCLUDED.denied,
        under_consideration = stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = stats.reviewed + EXCLUDED.reviewed;

    DELETE FROM arepl.submission_daily_reviewer_stats
    WHERE reviewer_id = p_secondary_user;

  ----- other deduplication

    DELETE FROM aredl.levels_created ac1
    USING aredl.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.level_id = ac2.level_id
        AND ac2.user_id = p_primary_user;

    DELETE FROM arepl.levels_created ac1
    USING arepl.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.level_id = ac2.level_id
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

	DELETE FROM aredl.bounty_completed bc1
	USING aredl.bounty_completed bc2
	WHERE bc1.user_id = p_secondary_user
		AND bc1.bounty_id = bc2.bounty_id
		AND bc2.user_id = p_primary_user;

	DELETE FROM arepl.bounty_completed bc1
	USING arepl.bounty_completed bc2
	WHERE bc1.user_id = p_secondary_user
		AND bc1.bounty_id = bc2.bounty_id
		AND bc2.user_id = p_primary_user;

    DELETE FROM oauth_connected_accounts ac1
    USING oauth_connected_accounts ac2
    WHERE ac1.user_id = p_secondary_user
      AND ac2.user_id = p_primary_user
      AND ac1.provider = ac2.provider;

  ----- change ownership

    UPDATE aredl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE aredl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE aredl.bounty_completed SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE aredl.submissions SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE aredl.submission_history SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE aredl.submissions_enabled SET moderator = p_primary_user WHERE moderator = p_secondary_user;
    UPDATE aredl.level_custom_copies SET added_by = p_primary_user WHERE added_by = p_secondary_user;
    UPDATE aredl.level_notes SET added_by = p_primary_user WHERE added_by = p_secondary_user;

    UPDATE arepl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE arepl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE arepl.bounty_completed SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE arepl.submissions SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE arepl.submission_history SET reviewer_id = p_primary_user WHERE reviewer_id = p_secondary_user;
    UPDATE arepl.submissions_enabled SET moderator = p_primary_user WHERE moderator = p_secondary_user;
    UPDATE arepl.level_custom_copies SET added_by = p_primary_user WHERE added_by = p_secondary_user;
    UPDATE arepl.level_notes SET added_by = p_primary_user WHERE added_by = p_secondary_user;

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

    PERFORM set_config('session_replication_role', 'origin', true);

  ----- log and delete

    INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
    SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
    FROM users WHERE id = p_secondary_user;

    UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

    DELETE FROM users WHERE id = p_secondary_user;
END;
$function$;

DROP MATERIALIZED VIEW public.clans_leaderboard;
DROP MATERIALIZED VIEW public.clans_created_levels;
DROP VIEW public.min_placement_clans_records;
DROP MATERIALIZED VIEW public.country_leaderboard;
DROP MATERIALIZED VIEW public.country_created_levels;
DROP VIEW public.min_placement_country_records;
DROP MATERIALIZED VIEW public.submission_totals;
DROP MATERIALIZED VIEW public.record_totals;
DROP MATERIALIZED VIEW public.user_leaderboard;
DROP VIEW public.badge_level_statistics;
DROP VIEW public.clan_member_points;
DROP VIEW public.user_pack_points;
DROP VIEW public.completed_packs;
DROP VIEW public.packs_points;
DROP INDEX public.users_country_id_idx;
DROP INDEX public.clan_members_clan_user_idx;

DROP FUNCTION public.submission_is_only_claim_toggle(public.submissions, public.submissions);
DROP TABLE public.bounties, public.bounty_completed, public.last_gddl_update, public.level_custom_copies, public.level_notes, public.level_updates, public.levels, public.levels_created, public.pack_levels, public.pack_tiers, public.packs, public.position_history, public.position_history_full_view, public.records, public.submission_daily_level_stats, public.submission_daily_reviewer_stats, public.submission_daily_total_stats, public.submission_history, public.submissions, public.submissions_enabled;
DROP FUNCTION public.append_position_history_full_view(target_list_id smallint, history_entry_i integer);
DROP FUNCTION public.apply_submission_daily_stats_diff(target_list_id smallint, p_day date, p_reviewer_id uuid, p_level_id uuid, p_submitted bigint, p_accepted bigint, p_denied bigint, p_under_consideration bigint);
DROP FUNCTION public.level_move();
DROP FUNCTION public.level_place();
DROP FUNCTION public.level_place_history();
DROP FUNCTION public.levels_points_after_insert();
DROP FUNCTION public.levels_points_before_insert();
DROP FUNCTION public.levels_points_before_update();
DROP FUNCTION public.max_list_pos(target_list_id smallint);
DROP FUNCTION public.max_list_pos_legacy(target_list_id smallint);
DROP FUNCTION public.point_formula(pos integer, level_count integer);
DROP FUNCTION public.rebuild_position_history_full_view(target_list_id smallint);
DROP FUNCTION public.rebuild_submission_daily_stats(target_list_id smallint);
DROP FUNCTION public.recalculate_points(target_list_id smallint);
DROP FUNCTION public.submission_log_history();
DROP FUNCTION public.submission_sync_record();
DROP FUNCTION public.submission_updated_at();
DROP FUNCTION public.update_submission_daily_stats_from_submission();
DROP FUNCTION public.validate_position_insert();
DROP FUNCTION public.validate_position_update();
DROP FUNCTION public.keep_list_id();
DROP FUNCTION public.lock_level_list();
DROP TABLE public.lists;
DROP TYPE public.custom_id_status;
DROP TYPE public.custom_id_type;
DROP TYPE public.level_notes_type;
DROP TYPE public.level_update_type;

ANALYZE aredl.bounties;
ANALYZE aredl.bounty_completed;
ANALYZE aredl.last_gddl_update;
ANALYZE aredl.level_custom_copies;
ANALYZE aredl.level_notes;
ANALYZE aredl.level_updates;
ANALYZE aredl.levels;
ANALYZE aredl.levels_created;
ANALYZE aredl.pack_levels;
ANALYZE aredl.pack_tiers;
ANALYZE aredl.packs;
ANALYZE aredl.position_history;
ANALYZE aredl.position_history_full_view;
ANALYZE aredl.records;
ANALYZE aredl.submission_daily_level_stats;
ANALYZE aredl.submission_daily_reviewer_stats;
ANALYZE aredl.submission_daily_total_stats;
ANALYZE aredl.submission_history;
ANALYZE aredl.submissions;
ANALYZE aredl.submissions_enabled;
ANALYZE arepl.bounties;
ANALYZE arepl.bounty_completed;
ANALYZE arepl.last_gddl_update;
ANALYZE arepl.level_custom_copies;
ANALYZE arepl.level_notes;
ANALYZE arepl.level_updates;
ANALYZE arepl.levels;
ANALYZE arepl.levels_created;
ANALYZE arepl.pack_levels;
ANALYZE arepl.pack_tiers;
ANALYZE arepl.packs;
ANALYZE arepl.position_history;
ANALYZE arepl.position_history_full_view;
ANALYZE arepl.records;
ANALYZE arepl.submission_daily_level_stats;
ANALYZE arepl.submission_daily_reviewer_stats;
ANALYZE arepl.submission_daily_total_stats;
ANALYZE arepl.submission_history;
ANALYZE arepl.submissions;
ANALYZE arepl.submissions_enabled;