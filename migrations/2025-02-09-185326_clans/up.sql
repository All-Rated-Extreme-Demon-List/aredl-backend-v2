CREATE TABLE IF NOT EXISTS clans (
	id UUID DEFAULT uuid_generate_v4(),
	global_name VARCHAR NOT NULL,
	tag VARCHAR NOT NULL,
	description TEXT,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(id),
	UNIQUE(tag)
);

CREATE TABLE IF NOT EXISTS clan_members (
    id uuid DEFAULT uuid_generate_v4(),
	clan_id uuid NOT NULL REFERENCES clans(id) ON DELETE CASCADE ON UPDATE CASCADE,
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
	role INTEGER NOT NULL DEFAULT 0,
	created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(id),
	UNIQUE(clan_id, user_id)
);

CREATE MATERIALIZED VIEW aredl_clans_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT cm.clan_id, r.level_id
    FROM aredl_records r
    JOIN clan_members cm ON r.submitted_by = cm.user_id
	JOIN users u ON r.submitted_by = u.id
    WHERE u.ban_level = 0
),
level_points AS (
    SELECT 
		c.clan_id,
		COALESCE(SUM(l.points), 0)::INTEGER AS level_points
    FROM completed_levels c
    JOIN aredl_levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest_position AS (
    SELECT 
		c.clan_id, 
		MIN(l.position) AS position
    FROM completed_levels c
    JOIN aredl_levels l ON c.level_id = l.id
    GROUP BY c.clan_id
),
hardest AS (
    SELECT 
		hp.clan_id, 
		l.id AS level_id
    FROM hardest_position hp
    JOIN aredl_levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        clan_id,
        count(*) AS c
    FROM completed_levels 
    GROUP BY clan_id
)
SELECT 
    RANK() OVER (ORDER BY lp.level_points DESC)::INTEGER AS rank,
	RANK() OVER (ORDER BY COALESCE(lc.c, 0) DESC)::INTEGER AS extremes_rank,
	lp.*,
    h.level_id AS hardest,
    COALESCE(lc.c, 0)::INTEGER AS extremes
FROM level_points lp
LEFT JOIN hardest h ON h.clan_id = lp.clan_id
LEFT JOIN level_count lc ON lc.clan_id = lp.clan_id;

CREATE OR REPLACE VIEW aredl_min_placement_clans_records AS
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