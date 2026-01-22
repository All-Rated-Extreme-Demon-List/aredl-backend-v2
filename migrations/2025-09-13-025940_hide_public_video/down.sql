DROP TRIGGER IF EXISTS update_record_placement_on_insert ON aredl.records;
DROP TRIGGER IF EXISTS update_record_placement_on_update ON aredl.records;
DROP TRIGGER IF EXISTS update_record_time ON aredl.records;

DROP FUNCTION IF EXISTS aredl.update_record_placement() CASCADE;
DROP FUNCTION IF EXISTS aredl.update_record_time() CASCADE;

DROP TRIGGER IF EXISTS update_record_placement_on_insert ON arepl.records;
DROP TRIGGER IF EXISTS update_record_placement_on_update ON arepl.records;
DROP TRIGGER IF EXISTS update_record_time ON arepl.records;

DROP FUNCTION IF EXISTS arepl.update_record_placement() CASCADE;
DROP FUNCTION IF EXISTS arepl.update_record_time() CASCADE;

DROP VIEW aredl.min_placement_clans_records;
DROP VIEW arepl.min_placement_clans_records;
DROP VIEW aredl.min_placement_country_records;
DROP VIEW arepl.min_placement_country_records;


ALTER TABLE aredl.records DROP COLUMN IF EXISTS hide_video;
ALTER TABLE arepl.records DROP COLUMN IF EXISTS hide_video;

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
