CREATE VIEW aredl_completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc FROM aredl_pack_levels pl GROUP BY pl.pack_id
    )
    SELECT r.submitted_by AS user_id, pl.pack_id
    FROM aredl_records r
    JOIN aredl_pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(r.*) = pcl.lc;

CREATE VIEW aredl_user_pack_points AS
    SELECT cp.user_id, SUM(p.points)::INTEGER AS points
    FROM aredl_completed_packs cp
    JOIN aredl_packs_points p ON p.id = cp.pack_id
    GROUP BY cp.user_id;

CREATE VIEW aredl_user_leaderboard AS
WITH user_points AS (
	SELECT r.submitted_by AS user_id, u.country, SUM(l.points)::INTEGER + COALESCE(pp.points, 0) AS total_points, COALESCE(pp.points, 0) AS pack_points
	FROM aredl_records r
	JOIN aredl_levels l ON r.level_id = l.id
	JOIN users u ON r.submitted_by = u.id
	LEFT JOIN aredl_user_pack_points pp ON pp.user_id = r.submitted_by
	GROUP BY r.submitted_by, pp.points, u.country
),
hardest_position AS (
	SELECT
		r.submitted_by AS user_id,
		MIN(l.position) AS position
	FROM aredl_records r
	JOIN aredl_levels l ON r.level_id = l.id
	GROUP BY r.submitted_by
),
hardest AS (
	SELECT
		hp.user_id,
		l.id AS level_id
	FROM hardest_position hp
	JOIN aredl_levels l ON hp.position = l.position
)
SELECT
	RANK() OVER (ORDER BY up.total_points DESC) AS rank,
	RANK() OVER (PARTITION BY up.country ORDER BY up.total_points DESC) AS country_rank,
	up.*,
	h.level_id AS hardest
FROM user_points up
JOIN hardest h ON h.user_id = up.user_id;