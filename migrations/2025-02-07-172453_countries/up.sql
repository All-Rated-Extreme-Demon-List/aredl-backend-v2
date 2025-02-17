CREATE MATERIALIZED VIEW aredl_country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT u.country, r.level_id
    FROM aredl_records r
    JOIN users u ON r.submitted_by = u.id
    JOIN aredl_levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND u.country IS NOT NULL AND u.country <> 0
),
level_points AS (
    SELECT 
		c.country,
		COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN aredl_levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest_position AS (
    SELECT 
		c.country, 
		MIN(l.position) AS position
    FROM completed_levels c
    JOIN aredl_levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest AS (
    SELECT 
		hp.country, 
		l.id AS level_id
    FROM hardest_position hp
    JOIN aredl_levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        country,
        count(*) AS c
    FROM completed_levels 
    GROUP BY country
),
user_count AS (
	SELECT
		country,
		count(*) AS c
	FROM users
	WHERE ban_level = 0
	AND country IS NOT NULL AND country <> 0
	GROUP BY country
)
SELECT 
    RANK() OVER (ORDER BY lp.level_points DESC)::INTEGER AS rank,
	RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
	lp.*,
	COALESCE(uc.c, 0)::INTEGER AS members_count,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes
FROM level_points lp
LEFT JOIN hardest h ON h.country = lp.country
LEFT JOIN level_count lc ON lc.country = lp.country
LEFT JOIN user_count uc ON uc.country = lp.country;

CREATE OR REPLACE VIEW aredl_min_placement_country_records AS
    WITH subquery AS (
        SELECT
            r.*,
            u.country,
            row_number() over ( PARTITION BY r.level_id, u.country ORDER BY r.placement_order) as order_pos
        FROM aredl_records r
        JOIN users u ON u.id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;