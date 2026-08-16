DROP MATERIALIZED VIEW aredl.user_leaderboard;
DROP MATERIALIZED VIEW arepl.user_leaderboard;

DROP VIEW aredl.user_pack_points;
DROP VIEW arepl.user_pack_points;

DROP VIEW aredl.completed_packs;
DROP VIEW arepl.completed_packs;


CREATE VIEW aredl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc
        FROM aredl.pack_levels pl
        GROUP BY pl.pack_id
    )
    SELECT
        r.submitted_by AS user_id,
        pl.pack_id
    FROM aredl.records r
    JOIN aredl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(*) = pcl.lc;

CREATE VIEW arepl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc
        FROM arepl.pack_levels pl
        GROUP BY pl.pack_id
    )
    SELECT
        r.submitted_by AS user_id,
        pl.pack_id
    FROM arepl.records r
    JOIN arepl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(*) = pcl.lc;

CREATE VIEW aredl.user_pack_points AS
    SELECT cp.user_id, SUM(p.points)::INTEGER AS points
    FROM aredl.completed_packs cp
    JOIN aredl.packs_points p ON p.id = cp.pack_id
    GROUP BY cp.user_id;

CREATE VIEW arepl.user_pack_points AS
    SELECT cp.user_id, SUM(p.points)::INTEGER AS points
    FROM arepl.completed_packs cp
    JOIN arepl.packs_points p ON p.id = cp.pack_id
    GROUP BY cp.user_id;

CREATE MATERIALIZED VIEW aredl.user_leaderboard AS
WITH user_points AS (
    SELECT
        u.id AS user_id,
        u.country,
        (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::INTEGER AS total_points,
        COALESCE(pp.points, 0)::INTEGER AS pack_points
    FROM users u
    LEFT JOIN aredl.records r ON u.id = r.submitted_by
    LEFT JOIN aredl.levels l
      ON r.level_id = l.id
     AND l.status = 'MainList'
    LEFT JOIN aredl.user_pack_points pp ON pp.user_id = r.submitted_by
    WHERE u.ban_level = 0
    GROUP BY u.id, u.country, pp.points
),
hardest_position AS (
    SELECT
        r.submitted_by AS user_id,
        MIN(l.position) AS position
    FROM aredl.records r
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE l.status = 'MainList'
    GROUP BY r.submitted_by
),
hardest AS (
    SELECT
        hp.user_id,
        l.id AS level_id
    FROM hardest_position hp
    JOIN aredl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT
        r.submitted_by AS id,
        COUNT(*) AS c
    FROM aredl.records r
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE l.status IN ('MainList', 'Pending')
    GROUP BY submitted_by
)
SELECT
    RANK() OVER (ORDER BY up.total_points DESC)::INTEGER AS rank,
    RANK() OVER (ORDER BY up.total_points - up.pack_points DESC)::INTEGER AS raw_rank,
    RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY up.total_points DESC)::INTEGER AS country_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY up.total_points - up.pack_points DESC)::INTEGER AS country_raw_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS country_extremes_rank,
    up.*,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes,
    cm.clan_id
FROM user_points up
LEFT JOIN hardest h ON h.user_id = up.user_id
LEFT JOIN level_count lc ON lc.id = up.user_id
LEFT JOIN clan_members cm ON cm.user_id = up.user_id;

CREATE MATERIALIZED VIEW arepl.user_leaderboard AS
WITH user_points AS (
    SELECT
        u.id AS user_id,
        u.country,
        (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::INTEGER AS total_points,
        COALESCE(pp.points, 0)::INTEGER AS pack_points
    FROM users u
    LEFT JOIN arepl.records r ON u.id = r.submitted_by
    LEFT JOIN arepl.levels l
      ON r.level_id = l.id
     AND l.status = 'MainList'
    LEFT JOIN arepl.user_pack_points pp ON pp.user_id = r.submitted_by
    WHERE u.ban_level = 0
    GROUP BY u.id, u.country, pp.points
),
hardest_position AS (
    SELECT
        r.submitted_by AS user_id,
        MIN(l.position) AS position
    FROM arepl.records r
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE l.status = 'MainList'
    GROUP BY r.submitted_by
),
hardest AS (
    SELECT
        hp.user_id,
        l.id AS level_id
    FROM hardest_position hp
    JOIN arepl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT
        r.submitted_by AS id,
        COUNT(*) AS c
    FROM arepl.records r
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE l.status IN ('MainList', 'Pending')
    GROUP BY submitted_by
)
SELECT
    RANK() OVER (ORDER BY up.total_points DESC)::INTEGER AS rank,
    RANK() OVER (ORDER BY up.total_points - up.pack_points DESC)::INTEGER AS raw_rank,
    RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY up.total_points DESC)::INTEGER AS country_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY up.total_points - up.pack_points DESC)::INTEGER AS country_raw_rank,
    RANK() OVER (PARTITION BY up.country ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS country_extremes_rank,
    up.*,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes,
    cm.clan_id
FROM user_points up
LEFT JOIN hardest h ON h.user_id = up.user_id
LEFT JOIN level_count lc ON lc.id = up.user_id
LEFT JOIN clan_members cm ON cm.user_id = up.user_id;