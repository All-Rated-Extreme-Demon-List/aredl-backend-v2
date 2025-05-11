CREATE SCHEMA IF NOT EXISTS aredl;

-- Cleanup

DROP TRIGGER IF EXISTS aredl_level_place            ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_level_place_history    ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_level_move             ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_validate_position_insert ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_validate_position_update ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_levels_points_before_insert  ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_levels_points_before_update  ON aredl_levels;
DROP TRIGGER IF EXISTS aredl_levels_points_after_insert   ON aredl_levels;

DROP TRIGGER IF EXISTS update_aredl_record_time                     ON aredl_records;
DROP TRIGGER IF EXISTS update_aredl_record_placement_on_update     ON aredl_records;
DROP TRIGGER IF EXISTS update_aredl_record_placement_on_insert     ON aredl_records;

DROP FUNCTION IF EXISTS aredl_point_formula(int, int) CASCADE;
DROP FUNCTION IF EXISTS aredl_level_place() CASCADE;
DROP FUNCTION IF EXISTS aredl_level_place_history() CASCADE;
DROP FUNCTION IF EXISTS aredl_level_move() CASCADE;
DROP FUNCTION IF EXISTS aredl_max_list_pos() CASCADE;
DROP FUNCTION IF EXISTS aredl_max_list_pos_legacy() CASCADE;
DROP FUNCTION IF EXISTS aredl_validate_position_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl_validate_position_update() CASCADE;
DROP FUNCTION IF EXISTS aredl_levels_points_before_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl_levels_points_before_update() CASCADE;
DROP FUNCTION IF EXISTS aredl_recalculate_points() CASCADE;
DROP FUNCTION IF EXISTS aredl_levels_points_after_insert() CASCADE;
DROP FUNCTION IF EXISTS update_aredl_record_time() CASCADE;
DROP FUNCTION IF EXISTS update_aredl_record_placement() CASCADE;

DROP MATERIALIZED VIEW IF EXISTS aredl_position_history_full_view;
DROP MATERIALIZED VIEW IF EXISTS aredl_user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl_clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl_country_leaderboard;
DROP VIEW IF EXISTS aredl_submissions_with_priority;
DROP VIEW IF EXISTS aredl_min_placement_country_records;
DROP VIEW IF EXISTS aredl_min_placement_clans_records;
DROP VIEW IF EXISTS aredl_user_pack_points;
DROP VIEW IF EXISTS aredl_completed_packs;
DROP VIEW IF EXISTS aredl_packs_points;

-- AREDL Schema

---- Tables

CREATE TABLE aredl.last_gddl_update (LIKE aredl_last_gddl_update INCLUDING ALL);
CREATE TABLE aredl.levels (LIKE aredl_levels INCLUDING ALL);
CREATE TABLE aredl.levels_created (LIKE aredl_levels_created INCLUDING ALL);
CREATE TABLE aredl.packs (LIKE aredl_packs INCLUDING ALL);
CREATE TABLE aredl.pack_tiers (LIKE aredl_pack_tiers INCLUDING ALL);
CREATE TABLE aredl.pack_levels (LIKE aredl_pack_levels INCLUDING ALL);
CREATE TABLE aredl.position_history (LIKE aredl_position_history INCLUDING ALL);
CREATE TABLE aredl.records (LIKE aredl_records INCLUDING ALL);
CREATE TABLE aredl.shifts (LIKE aredl_shifts INCLUDING ALL);
CREATE TABLE aredl.recurrent_shifts (LIKE aredl_recurrent_shifts INCLUDING ALL);
CREATE TABLE aredl.submissions (LIKE aredl_submissions INCLUDING ALL);
CREATE TABLE aredl.submission_history (LIKE submission_history INCLUDING ALL);

CREATE SEQUENCE aredl.position_history_i_seq
  OWNED BY aredl.position_history.i;

ALTER TABLE aredl.position_history
  ALTER COLUMN i SET DEFAULT nextval('aredl.position_history_i_seq');


---- Clean previous tables
DROP TABLE IF EXISTS aredl_last_gddl_update;
DROP TABLE IF EXISTS aredl_pack_levels;
DROP TABLE IF EXISTS aredl_packs;
DROP TABLE IF EXISTS aredl_pack_tiers;
DROP TABLE IF EXISTS aredl_position_history;
DROP TABLE IF EXISTS aredl_recurrent_shifts;
DROP TABLE IF EXISTS aredl_shifts;
DROP TABLE IF EXISTS submission_history;
DROP TABLE IF EXISTS aredl_submissions;
DROP TABLE IF EXISTS aredl_records;
DROP TABLE IF EXISTS aredl_levels_created;
DROP TABLE IF EXISTS aredl_levels;

DROP SEQUENCE IF EXISTS aredl_position_history_i_seq CASCADE;

---- Triggers/functions redefinitions

CREATE FUNCTION aredl.point_formula(pos int, level_count int) RETURNS int AS
$$
DECLARE
    a float;
    b float;
BEGIN
    IF pos > level_count THEN
        return 0;
    END IF;
    IF level_count <= 1 THEN
        return 500;
    END IF;
    b := (level_count - 1) * 0.0005832492374192;
    a := 6000 * sqrt(b);
    return ROUND((a / sqrt((CAST(pos AS float) - 1) / 50 + b) - 1000));
END
$$ LANGUAGE plpgsql;

CREATE FUNCTION aredl.level_place() RETURNS TRIGGER AS
$$
BEGIN
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
BEGIN
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
    move_dir int;
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
BEGIN
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
BEGIN
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


CREATE FUNCTION aredl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_record_time
BEFORE UPDATE ON aredl.records
FOR EACH ROW
EXECUTE FUNCTION aredl.update_record_time();

CREATE FUNCTION aredl.update_record_placement()
RETURNS TRIGGER AS $$
  BEGIN
    UPDATE aredl.records
    SET placement_order = sub.row_num - 1
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY level_id ORDER BY created_at) AS row_num
        FROM aredl.records
        WHERE EXISTS (
            SELECT 1 FROM new_table as n WHERE n.level_id = aredl.records.level_id
        )
    ) AS sub
    WHERE aredl.records.id = sub.id;
    RETURN null;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_record_placement_on_update
AFTER UPDATE ON aredl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION aredl.update_record_placement();

CREATE TRIGGER update_record_placement_on_insert
AFTER INSERT ON aredl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION aredl.update_record_placement();

---- Views

CREATE VIEW aredl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc FROM aredl.pack_levels pl GROUP BY pl.pack_id
    )
    SELECT r.submitted_by AS user_id, pl.pack_id
    FROM aredl.records r
    JOIN aredl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(r.*) = pcl.lc;

CREATE VIEW aredl.packs_points AS
    SELECT p.*, ROUND(SUM(l.points) * 0.5)::INTEGER AS points
    FROM aredl.packs p
    JOIN aredl.pack_levels pl ON p.id = pl.pack_id
    JOIN aredl.levels l ON l.id = pl.level_id
    GROUP BY p.id;

CREATE VIEW aredl.user_pack_points AS
    SELECT cp.user_id, SUM(p.points)::INTEGER AS points
    FROM aredl.completed_packs cp
    JOIN aredl.packs_points p ON p.id = cp.pack_id
    GROUP BY cp.user_id;

CREATE VIEW aredl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.placement_order
        ) AS order_pos
    FROM aredl.records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW aredl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.placement_order) as order_pos
        FROM aredl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE OR REPLACE VIEW aredl.submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl.submissions;

---- Materialized Views

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

-- Other 
CREATE OR REPLACE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid) RETURNS void AS
$$
BEGIN
	IF p_primary_user = p_secondary_user THEN
		RAISE EXCEPTION 'Cannot merge a user with themselves';
	END IF;

    IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_primary_user) THEN
        RAISE EXCEPTION 'Primary user % does not exist', p_primary_user;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_secondary_user) THEN
        RAISE EXCEPTION 'Secondary user % does not exist', p_secondary_user;
    END IF;

	DELETE FROM aredl.records ar1
	USING aredl.records ar2
	WHERE ar1.submitted_by = p_secondary_user
	AND ar1.level_id = ar2.level_id
	AND ar2.submitted_by = p_primary_user;

	DELETE FROM aredl.submissions as1
	USING aredl.submissions as2
	WHERE as1.submitted_by = p_secondary_user
	AND as1.level_id = as2.level_id
	AND as2.submitted_by = p_primary_user;

	DELETE FROM aredl.levels_created ac1
	USING aredl.levels_created ac2
	WHERE ac1.user_id = p_secondary_user
	AND ac1.level_id = ac2.level_id
	AND ac2.user_id = p_primary_user;

	DELETE FROM clan_members cm1
	USING clan_members cm2
	WHERE cm1.user_id = p_secondary_user
	AND cm2.user_id = p_primary_user;

	DELETE FROM user_roles ur1
	USING user_roles ur2
	WHERE ur1.user_id = p_secondary_user
	AND ur1.role_id = ur2.role_id
	AND ur2.user_id = p_primary_user;

	UPDATE aredl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE aredl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;

	INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
	SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
	FROM users WHERE id = p_secondary_user;

	UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

	DELETE FROM users WHERE id = p_secondary_user;

END;
$$ LANGUAGE plpgsql;
