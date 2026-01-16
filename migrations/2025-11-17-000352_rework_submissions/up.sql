ALTER TYPE submission_status ADD VALUE IF NOT EXISTS 'UnderReview';


-- Remove objects that depend on records

DROP TRIGGER update_record_placement_on_insert ON aredl.records;
DROP TRIGGER update_record_placement_on_update ON aredl.records;
DROP TRIGGER update_record_time ON aredl.records;

DROP FUNCTION aredl.update_record_placement() CASCADE;
DROP FUNCTION aredl.update_record_time() CASCADE;

DROP TRIGGER update_record_placement_on_insert ON arepl.records;
DROP TRIGGER update_record_placement_on_update ON arepl.records;
DROP TRIGGER update_record_time ON arepl.records;

DROP FUNCTION arepl.update_record_placement() CASCADE;
DROP FUNCTION arepl.update_record_time() CASCADE;

DROP VIEW aredl.min_placement_clans_records;
DROP VIEW arepl.min_placement_clans_records;
DROP VIEW aredl.min_placement_country_records;
DROP VIEW arepl.min_placement_country_records;



-- Make history entries include all submission fields

ALTER TABLE aredl.submissions
ADD COLUMN private_reviewer_notes text,
ADD COLUMN locked boolean NOT NULL DEFAULT FALSE;

ALTER TABLE arepl.submissions
ADD COLUMN private_reviewer_notes text,
ADD COLUMN locked boolean NOT NULL DEFAULT FALSE;

ALTER TABLE aredl.submission_history 
ADD COLUMN mobile boolean, 
ADD COLUMN ldm_id int, 
ADD COLUMN video_url varchar, 
ADD COLUMN raw_url varchar, 
ADD COLUMN mod_menu varchar, 
ADD COLUMN priority boolean,
ADD COLUMN private_reviewer_notes text,
ADD COLUMN locked boolean;

ALTER TABLE arepl.submission_history 
ADD COLUMN mobile boolean, 
ADD COLUMN ldm_id int, 
ADD COLUMN video_url varchar, 
ADD COLUMN raw_url varchar, 
ADD COLUMN mod_menu varchar, 
ADD COLUMN priority boolean, 
ADD COLUMN private_reviewer_notes text,
ADD COLUMN locked boolean,
ADD COLUMN completion_time bigint;



-- When multiple history entries refer to the same record id but with different submission ids because of updates, normalize them all to the same submission id

WITH per_record AS (
    SELECT DISTINCT ON (record_id)
        record_id,
        submission_id AS canonical_submission_id
    FROM aredl.submission_history
    WHERE record_id IS NOT NULL
    ORDER BY record_id, timestamp ASC, id ASC
),
sub_mapping AS (
    SELECT DISTINCT
        h.submission_id AS old_submission_id,
        p.canonical_submission_id
    FROM aredl.submission_history h
    JOIN per_record p USING (record_id)
    WHERE h.submission_id <> p.canonical_submission_id
)
UPDATE aredl.submission_history h
SET submission_id = m.canonical_submission_id
FROM sub_mapping m
WHERE h.submission_id = m.old_submission_id;

WITH per_record AS (
    SELECT DISTINCT ON (record_id)
        record_id,
        submission_id AS canonical_submission_id
    FROM arepl.submission_history
    WHERE record_id IS NOT NULL
    ORDER BY record_id, timestamp ASC, id ASC
),
sub_mapping AS (
    SELECT DISTINCT
        h.submission_id AS old_submission_id,
        p.canonical_submission_id
    FROM arepl.submission_history h
    JOIN per_record p USING (record_id)
    WHERE h.submission_id <> p.canonical_submission_id
)
UPDATE arepl.submission_history h
SET submission_id = m.canonical_submission_id
FROM sub_mapping m
WHERE h.submission_id = m.old_submission_id;



-- Create an accepted submission for existing records, if an history entry already exists for it, to keep the history ID consistent

WITH latest_history_per_level_user AS (
    SELECT h.submission_id, r.level_id, r.submitted_by, r.mobile, r.ldm_id, r.video_url, r.raw_url, r.user_notes, r.reviewer_notes, r.reviewer_id, r.created_at, r.updated_at,
		ROW_NUMBER() OVER (PARTITION BY r.level_id, r.submitted_by ORDER BY h.timestamp DESC, h.id DESC) AS rn
    FROM aredl.submission_history h
    JOIN aredl.records r ON r.id = h.record_id
    LEFT JOIN aredl.submissions s_by_id ON s_by_id.id = h.submission_id
    WHERE s_by_id.id IS NULL              -- skip if this submission_id already exists in submissions
),
candidates AS (
    SELECT * FROM latest_history_per_level_user
    WHERE rn = 1 -- keep latest per (level_id, submitted_by)
)
INSERT INTO aredl.submissions (id, level_id, submitted_by, mobile, ldm_id, video_url, raw_url, user_notes, reviewer_notes, reviewer_id, created_at, updated_at, status)
SELECT c.submission_id, c.level_id, c.submitted_by, c.mobile, c.ldm_id, c.video_url, c.raw_url, c.user_notes, c.reviewer_notes, c.reviewer_id, c.created_at, c.updated_at, 'Accepted'::submission_status
FROM candidates c
LEFT JOIN aredl.submissions s_pair
       ON s_pair.level_id = c.level_id
      AND s_pair.submitted_by = c.submitted_by
WHERE s_pair.id IS NULL; -- only insert if no (level_id, submitted_by) row exists yet


WITH latest_history_per_level_user AS (
    SELECT h.submission_id, r.level_id, r.submitted_by, r.mobile, r.ldm_id, r.video_url, r.raw_url, r.user_notes, r.reviewer_notes, r.reviewer_id, r.created_at, r.updated_at, r.completion_time,
           ROW_NUMBER() OVER (PARTITION BY r.level_id, r.submitted_by ORDER BY h.timestamp DESC, h.id DESC) AS rn
    FROM arepl.submission_history h
    JOIN arepl.records r ON r.id = h.record_id
    LEFT JOIN arepl.submissions s_by_id ON s_by_id.id = h.submission_id
    WHERE s_by_id.id IS NULL
),
candidates AS (
    SELECT * FROM latest_history_per_level_user WHERE rn = 1
)
INSERT INTO arepl.submissions (id, level_id, submitted_by, mobile, ldm_id, video_url, raw_url, user_notes, reviewer_notes, reviewer_id, created_at, updated_at, status, completion_time)
SELECT c.submission_id, c.level_id, c.submitted_by, c.mobile, c.ldm_id, c.video_url, c.raw_url, c.user_notes, c.reviewer_notes, c.reviewer_id, c.created_at, c.updated_at, 'Accepted'::submission_status, c.completion_time
FROM candidates c
LEFT JOIN arepl.submissions s_pair ON s_pair.level_id = c.level_id AND s_pair.submitted_by = c.submitted_by
WHERE s_pair.id IS NULL;



-- Create an accepted submission for records that don't have submission history at all

INSERT INTO aredl.submissions (id, level_id, submitted_by, mobile, ldm_id, video_url, raw_url, user_notes, reviewer_notes, reviewer_id, created_at, updated_at, status)
SELECT r.id, r.level_id, r.submitted_by, r.mobile, r.ldm_id, r.video_url, r.raw_url, r.user_notes, r.reviewer_notes, r.reviewer_id, r.created_at, r.updated_at, 'Accepted'::submission_status
FROM aredl.records r
LEFT JOIN aredl.submissions s_pair ON s_pair.level_id = r.level_id AND s_pair.submitted_by = r.submitted_by
LEFT JOIN aredl.submission_history h ON h.record_id = r.id
WHERE s_pair.id IS NULL AND h.submission_id IS NULL;

INSERT INTO arepl.submissions (id, level_id, submitted_by, mobile, ldm_id, video_url, raw_url, user_notes, reviewer_notes, reviewer_id, created_at, updated_at, status, completion_time)
SELECT r.id, r.level_id, r.submitted_by, r.mobile, r.ldm_id, r.video_url, r.raw_url, r.user_notes, r.reviewer_notes, r.reviewer_id, r.created_at, r.updated_at, 'Accepted'::submission_status, r.completion_time
FROM arepl.records r
LEFT JOIN arepl.submissions s_pair ON s_pair.level_id = r.level_id AND s_pair.submitted_by = r.submitted_by
LEFT JOIN arepl.submission_history h ON h.record_id = r.id
WHERE s_pair.id IS NULL AND h.submission_id IS NULL;



-- Create a corresponding history entry for those

INSERT INTO aredl.submission_history (id, submission_id, record_id, reviewer_notes, status, timestamp, user_notes, reviewer_id, video_url, raw_url, mobile, ldm_id)
SELECT uuid_generate_v4(), s.id, r.id, r.reviewer_notes, 'Accepted'::submission_status, r.created_at, r.user_notes, r.reviewer_id, r.video_url, r.raw_url, r.mobile, r.ldm_id
FROM aredl.records r
JOIN aredl.submissions s ON s.id = r.id
LEFT JOIN aredl.submission_history h ON h.submission_id = s.id
WHERE h.id IS NULL;

INSERT INTO arepl.submission_history (id, submission_id, record_id, reviewer_notes, status, timestamp, user_notes, reviewer_id, video_url, raw_url, mobile, ldm_id, completion_time)
SELECT uuid_generate_v4(), s.id, r.id, r.reviewer_notes, 'Accepted'::submission_status, r.created_at, r.user_notes, r.reviewer_id, r.video_url, r.raw_url, r.mobile, r.ldm_id, r.completion_time
FROM arepl.records r
JOIN arepl.submissions s ON s.id = r.id
LEFT JOIN arepl.submission_history h ON h.submission_id = s.id
WHERE h.id IS NULL;



-- Cleanup orphaned submission history entries

DELETE FROM aredl.submission_history AS sh
WHERE sh.submission_id IS NOT NULL
  AND NOT EXISTS (
        SELECT 1
        FROM aredl.submissions AS s
        WHERE s.id = sh.submission_id
  );

DELETE FROM arepl.submission_history AS sh
WHERE sh.submission_id IS NOT NULL
  AND NOT EXISTS (
        SELECT 1
        FROM arepl.submissions AS s
        WHERE s.id = sh.submission_id
  );



-- Update history and records schemas, and add constraints

ALTER TABLE aredl.submission_history
    DROP COLUMN record_id;

ALTER TABLE arepl.submission_history
    DROP COLUMN record_id;

ALTER TABLE aredl.submission_history
    ADD CONSTRAINT aredl_submission_history_submission_fk
        FOREIGN KEY (submission_id) REFERENCES aredl.submissions(id) ON DELETE CASCADE;

ALTER TABLE arepl.submission_history
    ADD CONSTRAINT arepl_submission_history_submission_fk
        FOREIGN KEY (submission_id) REFERENCES arepl.submissions(id) ON DELETE CASCADE;

ALTER TABLE aredl.records
    DROP COLUMN ldm_id,
    DROP COLUMN raw_url,
	DROP COLUMN user_notes,
	DROP COLUMN reviewer_notes,
    DROP COLUMN reviewer_id,
	DROP COLUMN placement_order,
	DROP COLUMN mod_menu,
    ADD COLUMN submission_id uuid REFERENCES aredl.submissions(id) ON DELETE CASCADE,
    ADD COLUMN achieved_at TIMESTAMPTZ;

UPDATE aredl.records
SET achieved_at = created_at;

ALTER TABLE aredl.records
    ALTER COLUMN achieved_at SET NOT NULL,
    ALTER COLUMN achieved_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.records
    DROP COLUMN ldm_id,
    DROP COLUMN raw_url,
	DROP COLUMN user_notes,
	DROP COLUMN reviewer_notes,
    DROP COLUMN reviewer_id,
	DROP COLUMN placement_order,
	DROP COLUMN mod_menu,
    ADD COLUMN submission_id uuid REFERENCES arepl.submissions(id) ON DELETE CASCADE,
    ADD COLUMN achieved_at TIMESTAMPTZ;

UPDATE arepl.records
SET achieved_at = created_at;

ALTER TABLE arepl.records
    ALTER COLUMN achieved_at SET NOT NULL,
    ALTER COLUMN achieved_at SET DEFAULT CLOCK_TIMESTAMP();

-- Records should reference the corresponding submission

UPDATE aredl.records AS r
SET submission_id = s.id
FROM aredl.submissions AS s
WHERE r.level_id = s.level_id
  AND r.submitted_by = s.submitted_by;

UPDATE arepl.records AS r
SET submission_id = s.id
FROM arepl.submissions AS s
WHERE r.level_id = s.level_id
  AND r.submitted_by = s.submitted_by;

ALTER TABLE aredl.records
    ALTER COLUMN submission_id SET NOT NULL;

ALTER TABLE arepl.records
    ALTER COLUMN submission_id SET NOT NULL;

-- Create triggers to sync accepted submissions with records

CREATE OR REPLACE FUNCTION aredl.submission_sync_record()
RETURNS TRIGGER AS
$$
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

    ELSIF NEW.status = 'Denied'
      AND (OLD.status IS DISTINCT FROM NEW.status)
    THEN
        DELETE FROM aredl.records
        WHERE submission_id = NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_sync_record_ins
AFTER INSERT ON aredl.submissions
FOR EACH ROW EXECUTE FUNCTION aredl.submission_sync_record();

CREATE TRIGGER submission_sync_record_upd
AFTER UPDATE OF status, mobile, video_url ON aredl.submissions
FOR EACH ROW EXECUTE FUNCTION aredl.submission_sync_record();

CREATE OR REPLACE FUNCTION arepl.submission_sync_record()
RETURNS TRIGGER AS
$$
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

    ELSIF NEW.status = 'Denied'
      AND (OLD.status IS DISTINCT FROM NEW.status)
    THEN
        DELETE FROM arepl.records
        WHERE submission_id = NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_sync_record_ins
AFTER INSERT ON arepl.submissions
FOR EACH ROW EXECUTE FUNCTION arepl.submission_sync_record();

CREATE TRIGGER submission_sync_record_upd
AFTER UPDATE OF status, mobile, video_url, completion_time ON arepl.submissions
FOR EACH ROW EXECUTE FUNCTION arepl.submission_sync_record();



-- Create triggers to populate submission history on submission changes

CREATE OR REPLACE FUNCTION aredl.submission_log_history()
RETURNS TRIGGER AS
$$
DECLARE
    only_claim_toggle boolean;
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, ldm_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.ldm_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        -- skip history if we only flip Pending <-> Claimed and reviewer and nothing else changes
        only_claim_toggle = NEW.status <> OLD.status
            AND ((OLD.status = 'Pending' AND NEW.status = 'Claimed') OR (OLD.status = 'Claimed' AND NEW.status = 'Pending'))
            AND NEW.user_notes IS NOT DISTINCT FROM OLD.user_notes
            AND NEW.reviewer_notes IS NOT DISTINCT FROM OLD.reviewer_notes
            AND NEW.private_reviewer_notes IS NOT DISTINCT FROM OLD.private_reviewer_notes
            AND NEW.locked IS NOT DISTINCT FROM OLD.locked
            AND NEW.mobile IS NOT DISTINCT FROM OLD.mobile
            AND NEW.ldm_id IS NOT DISTINCT FROM OLD.ldm_id
            AND NEW.video_url IS NOT DISTINCT FROM OLD.video_url
            AND NEW.raw_url IS NOT DISTINCT FROM OLD.raw_url
            AND NEW.mod_menu IS NOT DISTINCT FROM OLD.mod_menu
            AND NEW.priority IS NOT DISTINCT FROM OLD.priority;

        IF only_claim_toggle THEN
            RETURN NEW;
        END IF;

        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, ldm_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.ldm_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_log_history_ins AFTER INSERT ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_log_history();
CREATE TRIGGER submission_log_history_upd AFTER UPDATE ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_log_history();

CREATE OR REPLACE FUNCTION arepl.submission_log_history()
RETURNS TRIGGER AS
$$
DECLARE
    only_claim_toggle boolean;
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, ldm_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.ldm_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        only_claim_toggle = NEW.status <> OLD.status
            AND ((OLD.status = 'Pending' AND NEW.status = 'Claimed') OR (OLD.status = 'Claimed' AND NEW.status = 'Pending'))
            AND NEW.user_notes IS NOT DISTINCT FROM OLD.user_notes
            AND NEW.reviewer_notes IS NOT DISTINCT FROM OLD.reviewer_notes
            AND NEW.private_reviewer_notes IS NOT DISTINCT FROM OLD.private_reviewer_notes
            AND NEW.locked IS NOT DISTINCT FROM OLD.locked
            AND NEW.mobile IS NOT DISTINCT FROM OLD.mobile
            AND NEW.ldm_id IS NOT DISTINCT FROM OLD.ldm_id
            AND NEW.video_url IS NOT DISTINCT FROM OLD.video_url
            AND NEW.raw_url IS NOT DISTINCT FROM OLD.raw_url
            AND NEW.mod_menu IS NOT DISTINCT FROM OLD.mod_menu
            AND NEW.priority IS NOT DISTINCT FROM OLD.priority
            AND NEW.completion_time IS NOT DISTINCT FROM OLD.completion_time;

        IF only_claim_toggle THEN
            RETURN NEW;
        END IF;

        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, ldm_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.ldm_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_log_history_ins AFTER INSERT ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_log_history();
CREATE TRIGGER submission_log_history_upd AFTER UPDATE ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_log_history();



-- Create triggers to set updated_at when something changes

CREATE OR REPLACE FUNCTION aredl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        NEW.updated_at = CLOCK_TIMESTAMP();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_updated_at BEFORE UPDATE ON aredl.submissions FOR EACH ROW EXECUTE FUNCTION aredl.submission_updated_at();

CREATE OR REPLACE FUNCTION arepl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        NEW.updated_at = CLOCK_TIMESTAMP();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_updated_at BEFORE UPDATE ON arepl.submissions FOR EACH ROW EXECUTE FUNCTION arepl.submission_updated_at();



-- Recreate objects that depended on records

CREATE VIEW aredl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.achieved_at) as order_pos
        FROM aredl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE VIEW arepl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.achieved_at) as order_pos
        FROM arepl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE VIEW aredl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.achieved_at
        ) AS order_pos
    FROM aredl.records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW arepl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.achieved_at
        ) AS order_pos
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;