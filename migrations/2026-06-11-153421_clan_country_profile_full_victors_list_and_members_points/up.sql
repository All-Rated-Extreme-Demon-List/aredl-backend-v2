DROP VIEW IF EXISTS arepl.min_placement_country_records;
DROP VIEW IF EXISTS aredl.min_placement_country_records;
DROP VIEW IF EXISTS arepl.min_placement_clans_records;
DROP VIEW IF EXISTS aredl.min_placement_clans_records;

CREATE VIEW aredl.min_placement_clans_records AS
WITH subquery AS (
    SELECT
        r.*,
        cm.clan_id,
        row_number() OVER (
            PARTITION BY r.level_id, cm.clan_id
            ORDER BY r.achieved_at
        ) AS order_pos,
        count(*) OVER (
            PARTITION BY r.level_id, cm.clan_id
        ) AS completion_count
    FROM aredl.records r
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN aredl.levels l ON l.id = r.level_id
    WHERE l.status != 'Removed'
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW arepl.min_placement_clans_records AS
WITH subquery AS (
    SELECT
        r.*,
        cm.clan_id,
        row_number() OVER (
            PARTITION BY r.level_id, cm.clan_id
            ORDER BY r.achieved_at
        ) AS order_pos,
        count(*) OVER (
            PARTITION BY r.level_id, cm.clan_id
        ) AS completion_count
    FROM arepl.records r
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN arepl.levels l ON l.id = r.level_id
    WHERE l.status != 'Removed'
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
        ) AS order_pos,
        count(*) OVER (
            PARTITION BY r.level_id, u.country
        ) AS completion_count
    FROM aredl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN aredl.levels l ON l.id = r.level_id
    WHERE u.country IS NOT NULL
      AND l.status != 'Removed'
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
        ) AS order_pos,
        count(*) OVER (
            PARTITION BY r.level_id, u.country
        ) AS completion_count
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN arepl.levels l ON l.id = r.level_id
    WHERE u.country IS NOT NULL
      AND l.status != 'Removed'
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW aredl.clan_member_points AS
WITH clan_records AS (
    SELECT
        cm.clan_id,
        r.submitted_by,
        l.points,
        count(*) OVER (
            PARTITION BY cm.clan_id, r.level_id
        ) AS completion_count
    FROM aredl.records r
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN aredl.levels l ON l.id = r.level_id
    WHERE l.status != 'Removed'
)
SELECT
    clan_id,
    submitted_by,
    count(*) AS completed_levels,
    sum(points::double precision / completion_count::double precision) AS contributed_points
FROM clan_records
GROUP BY clan_id, submitted_by;

CREATE VIEW arepl.clan_member_points AS
WITH clan_records AS (
    SELECT
        cm.clan_id,
        r.submitted_by,
        l.points,
        count(*) OVER (
            PARTITION BY cm.clan_id, r.level_id
        ) AS completion_count
    FROM arepl.records r
    JOIN clan_members cm ON cm.user_id = r.submitted_by
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN arepl.levels l ON l.id = r.level_id
    WHERE l.status != 'Removed'
)
SELECT
    clan_id,
    submitted_by,
    count(*) AS completed_levels,
    sum(points::double precision / completion_count::double precision) AS contributed_points
FROM clan_records
GROUP BY clan_id, submitted_by;

CREATE VIEW aredl.country_member_points AS
WITH country_records AS (
    SELECT
        u.country,
        r.submitted_by,
        l.points,
        count(*) OVER (
            PARTITION BY u.country, r.level_id
        ) AS completion_count
    FROM aredl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN aredl.levels l ON l.id = r.level_id
    WHERE u.country IS NOT NULL
      AND l.status != 'Removed'
)
SELECT
    country,
    submitted_by,
    count(*) AS completed_levels,
    sum(points::double precision / completion_count::double precision) AS contributed_points
FROM country_records
GROUP BY country, submitted_by;

CREATE VIEW arepl.country_member_points AS
WITH country_records AS (
    SELECT
        u.country,
        r.submitted_by,
        l.points,
        count(*) OVER (
            PARTITION BY u.country, r.level_id
        ) AS completion_count
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by AND u.ban_level = 0
    JOIN arepl.levels l ON l.id = r.level_id
    WHERE u.country IS NOT NULL
      AND l.status != 'Removed'
)
SELECT
    country,
    submitted_by,
    count(*) AS completed_levels,
    sum(points::double precision / completion_count::double precision) AS contributed_points
FROM country_records
GROUP BY country, submitted_by;
