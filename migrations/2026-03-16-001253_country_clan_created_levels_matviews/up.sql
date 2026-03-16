CREATE MATERIALIZED VIEW aredl.country_created_levels AS
WITH explicit_creators AS (
    SELECT
        u.country,
        l.id AS level_id,
        lc.user_id AS creator_id,
        l.position AS order_pos
    FROM aredl.levels_created lc
    JOIN aredl.levels l ON l.id = lc.level_id
    JOIN users u ON u.id = lc.user_id
    WHERE u.country IS NOT NULL
),
-- levels that do not have any explicitly stated creator(s), 
-- therefore for which the publisher should be considered the creator as well
published_without_creators AS (
    SELECT
        u.country,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM aredl.levels l
    JOIN users u ON u.id = l.publisher_id
    LEFT JOIN aredl.levels_created lc ON lc.level_id = l.id
    WHERE u.country IS NOT NULL
      AND lc.level_id IS NULL
)
SELECT country, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT country, level_id, creator_id, order_pos
FROM published_without_creators;

CREATE INDEX aredl_country_created_levels_country_idx
    ON aredl.country_created_levels (country, order_pos, level_id, creator_id);

CREATE MATERIALIZED VIEW aredl.clans_created_levels AS
WITH explicit_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        lc.user_id AS creator_id,
        l.position AS order_pos
    FROM aredl.levels_created lc
    JOIN aredl.levels l ON l.id = lc.level_id
    JOIN clan_members cm ON cm.user_id = lc.user_id
),
published_without_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM aredl.levels l
    JOIN clan_members cm ON cm.user_id = l.publisher_id
    LEFT JOIN aredl.levels_created lc ON lc.level_id = l.id
    WHERE lc.level_id IS NULL
)
SELECT clan_id, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT clan_id, level_id, creator_id, order_pos
FROM published_without_creators;

CREATE INDEX aredl_clans_created_levels_clan_idx
    ON aredl.clans_created_levels (clan_id, order_pos, level_id, creator_id);

CREATE MATERIALIZED VIEW arepl.country_created_levels AS
WITH explicit_creators AS (
    SELECT
        u.country,
        l.id AS level_id,
        lc.user_id AS creator_id,
        l.position AS order_pos
    FROM arepl.levels_created lc
    JOIN arepl.levels l ON l.id = lc.level_id
    JOIN users u ON u.id = lc.user_id
    WHERE u.country IS NOT NULL
),
published_without_creators AS (
    SELECT
        u.country,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM arepl.levels l
    JOIN users u ON u.id = l.publisher_id
    LEFT JOIN arepl.levels_created lc ON lc.level_id = l.id
    WHERE u.country IS NOT NULL
      AND lc.level_id IS NULL
)
SELECT country, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT country, level_id, creator_id, order_pos
FROM published_without_creators;

CREATE INDEX arepl_country_created_levels_country_idx
    ON arepl.country_created_levels (country, order_pos, level_id, creator_id);

CREATE MATERIALIZED VIEW arepl.clans_created_levels AS
WITH explicit_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        lc.user_id AS creator_id,
        l.position AS order_pos
    FROM arepl.levels_created lc
    JOIN arepl.levels l ON l.id = lc.level_id
    JOIN clan_members cm ON cm.user_id = lc.user_id
),
published_without_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM arepl.levels l
    JOIN clan_members cm ON cm.user_id = l.publisher_id
    LEFT JOIN arepl.levels_created lc ON lc.level_id = l.id
    WHERE lc.level_id IS NULL
)
SELECT clan_id, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT clan_id, level_id, creator_id, order_pos
FROM published_without_creators;

CREATE INDEX arepl_clans_created_levels_clan_idx
    ON arepl.clans_created_levels (clan_id, order_pos, level_id, creator_id);
