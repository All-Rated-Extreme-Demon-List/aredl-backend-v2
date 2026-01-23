CREATE OR REPLACE VIEW aredl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.created_at) as order_pos
        FROM aredl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
        JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE OR REPLACE VIEW arepl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.created_at) as order_pos
        FROM arepl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by 
        JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE OR REPLACE VIEW aredl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.created_at
        ) AS order_pos
    FROM aredl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE OR REPLACE VIEW arepl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.created_at
        ) AS order_pos
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
)
SELECT *
FROM subquery
WHERE order_pos = 1;