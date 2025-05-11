DROP VIEW IF EXISTS aredl_submissions_with_priority;
DROP VIEW IF EXISTS aredl_min_placement_clans_records;
DROP VIEW IF EXISTS aredl_min_placement_country_records;


ALTER TABLE aredl_submissions
	RENAME COLUMN rejection_reason TO reviewer_notes;

ALTER TABLE aredl_submissions
	RENAME COLUMN additional_notes TO user_notes;

ALTER TABLE aredl_submissions
	DROP COLUMN IF EXISTS is_update;


ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS reviewer_notes VARCHAR;

ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS mod_menu VARCHAR;

ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS user_notes VARCHAR;

CREATE OR REPLACE VIEW aredl_submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl_submissions;

CREATE VIEW aredl_min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.placement_order
        ) AS order_pos
    FROM aredl_records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW aredl_min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.placement_order) as order_pos
        FROM aredl_records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;