DROP MATERIALIZED VIEW IF EXISTS arepl.clans_created_levels;
DROP MATERIALIZED VIEW IF EXISTS arepl.country_created_levels;
DROP MATERIALIZED VIEW IF EXISTS aredl.clans_created_levels;
DROP MATERIALIZED VIEW IF EXISTS aredl.country_created_levels;
DROP MATERIALIZED VIEW IF EXISTS arepl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.position_history_full_view;
DROP MATERIALIZED VIEW IF EXISTS aredl.position_history_full_view;

DROP TRIGGER IF EXISTS levels_points_after_insert ON arepl.levels;
DROP TRIGGER IF EXISTS level_place_history ON arepl.levels;
DROP TRIGGER IF EXISTS level_move ON arepl.levels;
DROP TRIGGER IF EXISTS level_place ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON arepl.levels;
DROP TRIGGER IF EXISTS levels_points_before_insert ON arepl.levels;
DROP TRIGGER IF EXISTS levels_points_before_update ON arepl.levels;

DROP TRIGGER IF EXISTS levels_points_after_insert ON aredl.levels;
DROP TRIGGER IF EXISTS level_place_history ON aredl.levels;
DROP TRIGGER IF EXISTS level_move ON aredl.levels;
DROP TRIGGER IF EXISTS level_place ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON aredl.levels;
DROP TRIGGER IF EXISTS levels_points_before_insert ON aredl.levels;
DROP TRIGGER IF EXISTS levels_points_before_update ON aredl.levels;

DROP FUNCTION IF EXISTS arepl.recalculate_points() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_after_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_move() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_place() CASCADE;
DROP FUNCTION IF EXISTS arepl.validate_position_update() CASCADE;
DROP FUNCTION IF EXISTS arepl.validate_position_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_before_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_before_update() CASCADE;
DROP FUNCTION IF EXISTS arepl.max_list_pos_legacy() CASCADE;
DROP FUNCTION IF EXISTS arepl.max_list_pos() CASCADE;

DROP FUNCTION IF EXISTS aredl.recalculate_points() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_after_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_move() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_place() CASCADE;
DROP FUNCTION IF EXISTS aredl.validate_position_update() CASCADE;
DROP FUNCTION IF EXISTS aredl.validate_position_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_before_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_before_update() CASCADE;
DROP FUNCTION IF EXISTS aredl.max_list_pos_legacy() CASCADE;
DROP FUNCTION IF EXISTS aredl.max_list_pos() CASCADE;

ALTER TABLE arepl.levels
    ADD COLUMN legacy BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE arepl.levels
SET legacy = status = 'Legacy';

ALTER TABLE aredl.levels
    ADD COLUMN legacy BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE aredl.levels
SET legacy = status = 'Legacy';

ALTER TABLE arepl.position_history
    ADD COLUMN legacy BOOLEAN;

UPDATE arepl.position_history
SET legacy = CASE
    WHEN new_status = 'Legacy' THEN TRUE
    WHEN new_status = 'MainList' THEN FALSE
    ELSE NULL
END;

ALTER TABLE aredl.position_history
    ADD COLUMN legacy BOOLEAN;

UPDATE aredl.position_history
SET legacy = CASE
    WHEN new_status = 'Legacy' THEN TRUE
    WHEN new_status = 'MainList' THEN FALSE
    ELSE NULL
END;

ALTER TABLE arepl.position_history
    DROP COLUMN IF EXISTS old_status,
    DROP COLUMN IF EXISTS new_status;

ALTER TABLE aredl.position_history
    DROP COLUMN IF EXISTS old_status,
    DROP COLUMN IF EXISTS new_status;

ALTER TABLE arepl.levels
    DROP CONSTRAINT IF EXISTS arepl_levels_status_position_check,
    DROP COLUMN IF EXISTS requires_raw_footage,
    DROP COLUMN IF EXISTS status,
    ALTER COLUMN position SET NOT NULL;

ALTER TABLE aredl.levels
    DROP CONSTRAINT IF EXISTS aredl_levels_status_position_check,
    DROP COLUMN IF EXISTS requires_raw_footage,
    DROP COLUMN IF EXISTS status,
    ALTER COLUMN position SET NOT NULL;

CREATE FUNCTION aredl.level_place() RETURNS TRIGGER AS
$$
DECLARE
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM aredl.levels;

    IF other_levels_count <= 0 THEN
        RETURN null;
    END IF;

    UPDATE aredl.levels
    SET position = position + 1
    WHERE position >= NEW.position AND id <> NEW.id;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place
AFTER INSERT ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_place();

CREATE FUNCTION aredl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above uuid;
    below uuid;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM aredl.levels;

    IF other_levels_count <= 0 THEN
        RETURN null;
    END IF;

    above := (SELECT id FROM aredl.levels WHERE position = NEW.position - 1);
    below := (SELECT id FROM aredl.levels WHERE position = NEW.position + 1);

    INSERT INTO aredl.position_history(new_position, old_position, legacy, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NEW.legacy, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.level_place_history();

CREATE FUNCTION aredl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    legacy_history boolean;
    above uuid;
    below uuid;
BEGIN
    IF NEW.position = OLD.position AND NEW.legacy = OLD.legacy THEN
        RETURN null;
    END IF;

    UPDATE aredl.levels
    SET position = position + (CASE WHEN NEW.position < OLD.position THEN 1 ELSE -1 END)
    WHERE id <> NEW.id AND position
        BETWEEN LEAST(NEW.position, OLD.position)
        AND GREATEST(NEW.position, OLD.position);

    legacy_history := NULL;
    IF NEW.legacy <> OLD.legacy THEN
        legacy_history := NEW.legacy;
    END IF;

    above := (SELECT id FROM aredl.levels WHERE position = NEW.position - 1);
    below := (SELECT id FROM aredl.levels WHERE position = NEW.position + 1);

    INSERT INTO aredl.position_history(new_position, old_position, legacy, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, legacy_history, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF "position", "legacy" ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_move();

CREATE FUNCTION aredl.max_list_pos() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM aredl.levels WHERE legacy = false;
$$ LANGUAGE sql;

CREATE FUNCTION aredl.max_list_pos_legacy() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM aredl.levels;
$$ LANGUAGE sql;

CREATE FUNCTION aredl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM aredl.levels;

    IF other_levels_count <= 0 THEN
        RETURN new;
    END IF;

    IF NEW.legacy THEN
        highestPos := aredl.max_list_pos_legacy() + 1;
        lowestPos := aredl.max_list_pos() + 1;
    ELSE
        highestPos := aredl.max_list_pos() + 1;
        lowestPos := 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_insert
BEFORE INSERT ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.validate_position_insert();

CREATE FUNCTION aredl.validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM aredl.levels;

    IF other_levels_count <= 0 THEN
        RETURN new;
    END IF;

    IF NEW.legacy THEN
        IF NEW.legacy <> OLD.legacy THEN
            lowestPos := aredl.max_list_pos();
        ELSE
            lowestPos := aredl.max_list_pos() + 1;
        END IF;
        highestPos := aredl.max_list_pos_legacy();
    ELSE
        IF NEW.legacy <> OLD.legacy THEN
            highestPos := aredl.max_list_pos() + 1;
        ELSE
            highestPos := aredl.max_list_pos();
        END IF;
        lowestPos := 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_update
BEFORE UPDATE OF "position", "legacy" ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.validate_position_update();

CREATE FUNCTION aredl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    new.points := aredl.point_formula(new.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE legacy = false) AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF "position" ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.levels_points_before_update();

CREATE FUNCTION aredl.levels_points_before_insert() RETURNS TRIGGER AS
$$
BEGIN
    new.points := aredl.point_formula(new.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE legacy = false) + 1 AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_insert
BEFORE INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.levels_points_before_insert();

CREATE FUNCTION aredl.recalculate_points() RETURNS void AS
$$
BEGIN
    UPDATE aredl.levels
    SET points = aredl.point_formula(position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE legacy = false) AS INT));
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION aredl.levels_points_after_insert() RETURNS TRIGGER AS
$$
BEGIN
   PERFORM aredl.recalculate_points();
   RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_after_insert
AFTER INSERT ON aredl.levels
FOR EACH STATEMENT
EXECUTE PROCEDURE aredl.levels_points_after_insert();

CREATE FUNCTION arepl.level_place() RETURNS TRIGGER AS
$$
DECLARE
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM arepl.levels;

    IF other_levels_count <= 0 THEN
        RETURN null;
    END IF;

    UPDATE arepl.levels
    SET position = position + 1
    WHERE position >= NEW.position AND id <> NEW.id;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place
AFTER INSERT ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.level_place();

CREATE FUNCTION arepl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above uuid;
    below uuid;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM arepl.levels;

    IF other_levels_count <= 0 THEN
        RETURN null;
    END IF;

    above := (SELECT id FROM arepl.levels WHERE position = NEW.position - 1);
    below := (SELECT id FROM arepl.levels WHERE position = NEW.position + 1);

    INSERT INTO arepl.position_history(new_position, old_position, legacy, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NEW.legacy, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.level_place_history();

CREATE FUNCTION arepl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    legacy_history boolean;
    above uuid;
    below uuid;
BEGIN
    IF NEW.position = OLD.position AND NEW.legacy = OLD.legacy THEN
        RETURN null;
    END IF;

    UPDATE arepl.levels
    SET position = position + (CASE WHEN NEW.position < OLD.position THEN 1 ELSE -1 END)
    WHERE id <> NEW.id AND position
        BETWEEN LEAST(NEW.position, OLD.position)
        AND GREATEST(NEW.position, OLD.position);

    legacy_history := NULL;
    IF NEW.legacy <> OLD.legacy THEN
        legacy_history := NEW.legacy;
    END IF;

    above := (SELECT id FROM arepl.levels WHERE position = NEW.position - 1);
    below := (SELECT id FROM arepl.levels WHERE position = NEW.position + 1);

    INSERT INTO arepl.position_history(new_position, old_position, legacy, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, legacy_history, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF "position", "legacy" ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.level_move();

CREATE FUNCTION arepl.max_list_pos() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM arepl.levels WHERE legacy = false;
$$ LANGUAGE sql;

CREATE FUNCTION arepl.max_list_pos_legacy() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM arepl.levels;
$$ LANGUAGE sql;

CREATE FUNCTION arepl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM arepl.levels;

    IF other_levels_count <= 0 THEN
        RETURN new;
    END IF;

    IF NEW.legacy THEN
        highestPos := arepl.max_list_pos_legacy() + 1;
        lowestPos := arepl.max_list_pos() + 1;
    ELSE
        highestPos := arepl.max_list_pos() + 1;
        lowestPos := 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_insert
BEFORE INSERT ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.validate_position_insert();

CREATE FUNCTION arepl.validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
    other_levels_count int;
BEGIN
    SELECT COUNT(*) - 1
    INTO other_levels_count
    FROM arepl.levels;

    IF other_levels_count <= 0 THEN
        RETURN new;
    END IF;

    IF NEW.legacy THEN
        IF NEW.legacy <> OLD.legacy THEN
            lowestPos := arepl.max_list_pos();
        ELSE
            lowestPos := arepl.max_list_pos() + 1;
        END IF;
        highestPos := arepl.max_list_pos_legacy();
    ELSE
        IF NEW.legacy <> OLD.legacy THEN
            highestPos := arepl.max_list_pos() + 1;
        ELSE
            highestPos := arepl.max_list_pos();
        END IF;
        lowestPos := 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_update
BEFORE UPDATE OF "position", "legacy" ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.validate_position_update();

CREATE FUNCTION arepl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    new.points := arepl.point_formula(new.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE legacy = false) AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF "position" ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.levels_points_before_update();

CREATE FUNCTION arepl.levels_points_before_insert() RETURNS TRIGGER AS
$$
BEGIN
    new.points := arepl.point_formula(new.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE legacy = false) + 1 AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_insert
BEFORE INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.levels_points_before_insert();

CREATE FUNCTION arepl.recalculate_points() RETURNS void AS
$$
BEGIN
    UPDATE arepl.levels
    SET points = arepl.point_formula(position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE legacy = false) AS INT));
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION arepl.levels_points_after_insert() RETURNS TRIGGER AS
$$
BEGIN
   PERFORM arepl.recalculate_points();
   RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_after_insert
AFTER INSERT ON arepl.levels
FOR EACH STATEMENT
EXECUTE PROCEDURE arepl.levels_points_after_insert();

CREATE MATERIALIZED VIEW aredl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, new_position, old_position, legacy, created_at, affected_level
    FROM aredl.position_history
),
full_history AS (
	SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) as prev_pos, legacy, legacy AS prev_legacy, created_at AS action_at, affected_level AS cause, false AS moved
	FROM ranked_history
	WHERE old_position IS NULL
	UNION
	SELECT
		r.i,
		h.id,
		(CASE
			WHEN r.affected_level = h.id THEN r.new_position
			WHEN r.old_position IS NULL THEN
				CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
			WHEN r.new_position IS NULL THEN
				CASE WHEN h.position >= r.old_position THEN h.position - 1 ELSE h.position END
			WHEN r.old_position < r.new_position THEN
				CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
			ELSE
				CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
		END) as position,
		h.position AS prev_pos,
		(CASE WHEN r.affected_level = h.id AND r.legacy IS NOT NULL THEN r.legacy ELSE h.legacy END) AS legacy,
		h.legacy AS prev_legacy,
		r.created_at AS action_at,
		r.affected_level as cause,
		(r.old_position IS NOT NULL AND r.new_position IS NOT NULL) as moved
	FROM ranked_history r
	INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS (
    SELECT i::INTEGER as ord, id as affected_level, position, moved, legacy, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_legacy <> legacy OR prev_pos IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) as pos_diff FROM filtered;

CREATE MATERIALIZED VIEW aredl.user_leaderboard AS
WITH user_points AS (
	SELECT u.id AS user_id, u.country, (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::INTEGER AS total_points, (COALESCE(pp.points, 0))::INTEGER AS pack_points
	FROM users u
	LEFT JOIN aredl.records r ON u.id = r.submitted_by
	LEFT JOIN aredl.levels l ON r.level_id = l.id
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
	GROUP BY r.submitted_by
),
hardest AS (
	SELECT
		hp.user_id,
		l.id AS level_id
	FROM hardest_position hp
	JOIN aredl.levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        r.submitted_by AS id,
        count(*) AS c
    FROM aredl.records r
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE l.legacy = false
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
    WHERE u.ban_level = 0
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
    JOIN aredl.levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        clan_id,
        count(*) AS c
    FROM completed_levels
    GROUP BY clan_id
),
user_count AS (
	SELECT
		clan_id,
		count(*) AS c
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
      AND u.country IS NOT NULL AND u.country <> 0
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
    JOIN aredl.levels l ON hp.position = l.position
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

CREATE MATERIALIZED VIEW arepl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, new_position, old_position, legacy, created_at, affected_level
    FROM arepl.position_history
),
full_history AS (
	SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) as prev_pos, legacy, legacy AS prev_legacy, created_at AS action_at, affected_level AS cause, false AS moved
	FROM ranked_history
	WHERE old_position IS NULL
	UNION
	SELECT
		r.i,
		h.id,
		(CASE
			WHEN r.affected_level = h.id THEN r.new_position
			WHEN r.old_position IS NULL THEN
				CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
			WHEN r.new_position IS NULL THEN
				CASE WHEN h.position >= r.old_position THEN h.position - 1 ELSE h.position END
			WHEN r.old_position < r.new_position THEN
				CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
			ELSE
				CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
		END) as position,
		h.position AS prev_pos,
		(CASE WHEN r.affected_level = h.id AND r.legacy IS NOT NULL THEN r.legacy ELSE h.legacy END) AS legacy,
		h.legacy AS prev_legacy,
		r.created_at AS action_at,
		r.affected_level as cause,
		(r.old_position IS NOT NULL AND r.new_position IS NOT NULL) as moved
	FROM ranked_history r
	INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS (
    SELECT i::INTEGER as ord, id as affected_level, position, moved, legacy, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_legacy <> legacy OR prev_pos IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) as pos_diff FROM filtered;

CREATE MATERIALIZED VIEW arepl.user_leaderboard AS
WITH user_points AS (
	SELECT u.id AS user_id, u.country, (COALESCE(SUM(l.points), 0) + COALESCE(pp.points, 0))::INTEGER AS total_points, (COALESCE(pp.points, 0))::INTEGER AS pack_points
	FROM users u
	LEFT JOIN arepl.records r ON u.id = r.submitted_by
	LEFT JOIN arepl.levels l ON r.level_id = l.id
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
	GROUP BY r.submitted_by
),
hardest AS (
	SELECT
		hp.user_id,
		l.id AS level_id
	FROM hardest_position hp
	JOIN arepl.levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        r.submitted_by AS id,
        count(*) AS c
    FROM arepl.records r
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE l.legacy = false
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
    WHERE u.ban_level = 0
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
    JOIN arepl.levels l ON hp.position = l.position
),
level_count AS (
    SELECT
        clan_id,
        count(*) AS c
    FROM completed_levels
    GROUP BY clan_id
),
user_count AS (
	SELECT
		clan_id,
		count(*) AS c
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
      AND u.country IS NOT NULL AND u.country <> 0
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
    JOIN arepl.levels l ON hp.position = l.position
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

DROP TYPE IF EXISTS level_status;
