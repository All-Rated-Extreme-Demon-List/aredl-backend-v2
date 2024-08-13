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

CREATE MATERIALIZED VIEW aredl_user_leaderboard AS
WITH user_points AS (
	SELECT u.id AS user_id, u.country, u.discord_id, u.discord_avatar, (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::INTEGER AS total_points, (COALESCE(pp.points, 0))::INTEGER AS pack_points
	FROM users u
	LEFT JOIN aredl_records r ON u.id = r.submitted_by
	LEFT JOIN aredl_levels l ON r.level_id = l.id
	LEFT JOIN aredl_user_pack_points pp ON pp.user_id = r.submitted_by
	GROUP BY u.id, u.country, u.discord_id, u.discord_avatar, pp.points
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
),
level_count AS (
    SELECT
        r.submitted_by AS id,
        count(*) AS c
    FROM aredl_records r
    JOIN aredl_levels l ON r.level_id = l.id
    WHERE l.legacy = false
    GROUP BY submitted_by
)
SELECT
	RANK() OVER (ORDER BY up.total_points DESC)::INTEGER AS rank,
	RANK() OVER (PARTITION BY up.country ORDER BY up.total_points DESC)::INTEGER AS country_rank,
	up.*,
	h.level_id AS hardest,
	COALESCE(lc.c, 0)::INTEGER AS extremes
FROM user_points up
LEFT JOIN hardest h ON h.user_id = up.user_id
LEFT JOIN level_count lc ON lc.id = up.user_id;