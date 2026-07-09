DROP MATERIALIZED VIEW IF EXISTS aredl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.clans_leaderboard;

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

CREATE MATERIALIZED VIEW aredl.clans_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT cm.clan_id, r.level_id
    FROM aredl.records r
    JOIN clan_members cm ON r.submitted_by = cm.user_id
    JOIN users u ON r.submitted_by = u.id
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND l.status IN ('MainList', 'Pending')
),
level_points AS (
    SELECT
        c.clan_id,
        COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN aredl.levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest_position AS (
    SELECT
        c.clan_id,
        MIN(l.position) AS position
    FROM completed_levels c
    JOIN aredl.levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest AS (
    SELECT
        hp.clan_id,
        l.id AS level_id
    FROM hardest_position hp
    JOIN aredl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT clan_id, COUNT(*) AS c
    FROM completed_levels
    GROUP BY clan_id
),
user_count AS (
    SELECT clan_id, COUNT(*) AS c
    FROM clan_members
    GROUP BY clan_id
)
SELECT
    RANK() OVER (ORDER BY lp.level_points DESC)::INTEGER AS rank,
    RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
    lp.*,
    COALESCE(uc.c, 0)::INTEGER AS members_count,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes
FROM level_points lp
LEFT JOIN hardest h ON h.clan_id = lp.clan_id
LEFT JOIN level_count lc ON lc.clan_id = lp.clan_id
LEFT JOIN user_count uc ON uc.clan_id = lp.clan_id;

CREATE MATERIALIZED VIEW aredl.country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT u.country, r.level_id
    FROM aredl.records r
    JOIN users u ON r.submitted_by = u.id
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND u.country IS NOT NULL
      AND u.country <> 0
      AND l.status IN ('MainList', 'Pending')
),
level_points AS (
    SELECT
        c.country,
        COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN aredl.levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest_position AS (
    SELECT
        c.country,
        MIN(l.position) AS position
    FROM completed_levels c
    JOIN aredl.levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest AS (
    SELECT
        hp.country,
        l.id AS level_id
    FROM hardest_position hp
    JOIN aredl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT country, COUNT(*) AS c
    FROM completed_levels
    GROUP BY country
),
user_count AS (
    SELECT country, COUNT(*) AS c
    FROM users
    WHERE ban_level = 0
      AND country IS NOT NULL
      AND country <> 0
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

CREATE MATERIALIZED VIEW arepl.clans_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT cm.clan_id, r.level_id
    FROM arepl.records r
    JOIN clan_members cm ON r.submitted_by = cm.user_id
    JOIN users u ON r.submitted_by = u.id
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND l.status IN ('MainList', 'Pending')
),
level_points AS (
    SELECT
        c.clan_id,
        COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN arepl.levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest_position AS (
    SELECT
        c.clan_id,
        MIN(l.position) AS position
    FROM completed_levels c
    JOIN arepl.levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest AS (
    SELECT
        hp.clan_id,
        l.id AS level_id
    FROM hardest_position hp
    JOIN arepl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT clan_id, COUNT(*) AS c
    FROM completed_levels
    GROUP BY clan_id
),
user_count AS (
    SELECT clan_id, COUNT(*) AS c
    FROM clan_members
    GROUP BY clan_id
)
SELECT
    RANK() OVER (ORDER BY lp.level_points DESC)::INTEGER AS rank,
    RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
    lp.*,
    COALESCE(uc.c, 0)::INTEGER AS members_count,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes
FROM level_points lp
LEFT JOIN hardest h ON h.clan_id = lp.clan_id
LEFT JOIN level_count lc ON lc.clan_id = lp.clan_id
LEFT JOIN user_count uc ON uc.clan_id = lp.clan_id;

CREATE MATERIALIZED VIEW arepl.country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT u.country, r.level_id
    FROM arepl.records r
    JOIN users u ON r.submitted_by = u.id
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND u.country IS NOT NULL
      AND u.country <> 0
      AND l.status IN ('MainList', 'Pending')
),
level_points AS (
    SELECT
        c.country,
        COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN arepl.levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest_position AS (
    SELECT
        c.country,
        MIN(l.position) AS position
    FROM completed_levels c
    JOIN arepl.levels l ON c.level_id = l.id
    GROUP BY c.country
),
hardest AS (
    SELECT
        hp.country,
        l.id AS level_id
    FROM hardest_position hp
    JOIN arepl.levels l
      ON hp.position = l.position
     AND l.status = 'MainList'
),
level_count AS (
    SELECT country, COUNT(*) AS c
    FROM completed_levels
    GROUP BY country
),
user_count AS (
    SELECT country, COUNT(*) AS c
    FROM users
    WHERE ban_level = 0
      AND country IS NOT NULL
      AND country <> 0
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
