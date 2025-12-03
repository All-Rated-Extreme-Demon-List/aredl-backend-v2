DROP VIEW aredl.min_placement_clans_records;
DROP VIEW arepl.min_placement_clans_records;
DROP VIEW aredl.min_placement_country_records;
DROP VIEW arepl.min_placement_country_records;

DROP VIEW aredl.submissions_with_priority;
DROP VIEW arepl.submissions_with_priority;


DROP TRIGGER submission_sync_record_ins ON aredl.submissions;
DROP TRIGGER submission_sync_record_upd ON aredl.submissions;
DROP FUNCTION aredl.submission_sync_record();

DROP TRIGGER submission_sync_record_ins ON arepl.submissions;
DROP TRIGGER submission_sync_record_upd ON arepl.submissions;
DROP FUNCTION arepl.submission_sync_record();



DROP TRIGGER submission_log_history_ins ON aredl.submissions;
DROP TRIGGER submission_log_history_upd ON aredl.submissions;
DROP FUNCTION aredl.submission_log_history();

DROP TRIGGER submission_log_history_ins ON arepl.submissions;
DROP TRIGGER submission_log_history_upd ON arepl.submissions;
DROP FUNCTION arepl.submission_log_history();



DROP TRIGGER submission_updated_at ON aredl.submissions;
DROP FUNCTION aredl.submission_updated_at();

DROP TRIGGER submission_updated_at ON arepl.submissions;
DROP FUNCTION arepl.submission_updated_at();



ALTER TABLE aredl.submission_history DROP CONSTRAINT aredl_submission_history_submission_fk;
ALTER TABLE arepl.submission_history DROP CONSTRAINT arepl_submission_history_submission_fk;

ALTER TABLE aredl.submission_history ADD COLUMN record_id uuid;
ALTER TABLE arepl.submission_history ADD COLUMN record_id uuid;

ALTER TABLE aredl.submission_history
    DROP COLUMN mobile,
    DROP COLUMN ldm_id,
    DROP COLUMN video_url,
    DROP COLUMN raw_url,
    DROP COLUMN mod_menu,
    DROP COLUMN priority,
    DROP COLUMN private_reviewer_notes;

ALTER TABLE arepl.submission_history
    DROP COLUMN mobile,
    DROP COLUMN ldm_id,
    DROP COLUMN video_url,
    DROP COLUMN raw_url,
    DROP COLUMN mod_menu,
    DROP COLUMN priority,
    DROP COLUMN completion_time,
    DROP COLUMN private_reviewer_notes;

ALTER TABLE aredl.submissions
    DROP COLUMN private_reviewer_notes;

ALTER TABLE arepl.submissions
    DROP COLUMN private_reviewer_notes;


ALTER TABLE aredl.records
    ADD COLUMN ldm_id int,
    ADD COLUMN raw_url varchar,
    ADD COLUMN user_notes text,
    ADD COLUMN reviewer_notes text,
    ADD COLUMN reviewer_id uuid,
    ADD COLUMN placement_order int,
    ADD COLUMN mod_menu varchar,
    DROP COLUMN submission_id;

ALTER TABLE arepl.records
    ADD COLUMN ldm_id int,
    ADD COLUMN raw_url varchar,
    ADD COLUMN user_notes text,
    ADD COLUMN reviewer_notes text,
    ADD COLUMN reviewer_id uuid,
    ADD COLUMN placement_order int,
    ADD COLUMN mod_menu varchar,
    DROP COLUMN submission_id;




CREATE FUNCTION aredl.update_record_placement()
RETURNS TRIGGER AS $$
  BEGIN
    UPDATE aredl.records
    SET placement_order = sub.row_num - 1
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY level_id ORDER BY created_at) AS row_num
        FROM aredl.records
        WHERE EXISTS (
            SELECT 1 FROM new_table as n WHERE n.level_id = aredl.records.level_id
        )
    ) AS sub
    WHERE aredl.records.id = sub.id;
    RETURN null;
  END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION arepl.update_record_placement()
RETURNS TRIGGER AS $$
  BEGIN
    UPDATE arepl.records
    SET placement_order = sub.row_num - 1
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY level_id ORDER BY created_at) AS row_num
        FROM arepl.records
        WHERE EXISTS (
            SELECT 1 FROM new_table as n WHERE n.level_id = arepl.records.level_id
        )
    ) AS sub
    WHERE arepl.records.id = sub.id;
    RETURN null;
  END;
$$ LANGUAGE plpgsql;


CREATE TRIGGER update_record_placement_on_update
AFTER UPDATE ON aredl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION aredl.update_record_placement();
CREATE TRIGGER update_record_placement_on_insert
AFTER INSERT ON aredl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION aredl.update_record_placement();

CREATE TRIGGER update_record_placement_on_update
AFTER UPDATE ON arepl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION arepl.update_record_placement();

CREATE TRIGGER update_record_placement_on_insert
AFTER INSERT ON arepl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION arepl.update_record_placement();


CREATE OR REPLACE FUNCTION aredl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
  IF (to_jsonb(NEW) - 'updated_at' - 'placement_order')
       IS DISTINCT FROM
     (to_jsonb(OLD) - 'updated_at' - 'placement_order')
  THEN
    NEW.updated_at := now();
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
  IF (to_jsonb(NEW) - 'updated_at' - 'placement_order')
       IS DISTINCT FROM
     (to_jsonb(OLD) - 'updated_at' - 'placement_order')
  THEN
    NEW.updated_at := now();
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;



CREATE TRIGGER update_record_time
BEFORE UPDATE ON aredl.records
FOR EACH ROW
EXECUTE FUNCTION aredl.update_record_time();

CREATE TRIGGER update_record_time
BEFORE UPDATE ON arepl.records
FOR EACH ROW
EXECUTE FUNCTION arepl.update_record_time();



CREATE VIEW aredl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.placement_order) as order_pos
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
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.placement_order) as order_pos
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
          ORDER BY r.placement_order
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
          ORDER BY r.placement_order
        ) AS order_pos
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE OR REPLACE VIEW aredl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl.submissions;

CREATE OR REPLACE VIEW arepl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM arepl.submissions;