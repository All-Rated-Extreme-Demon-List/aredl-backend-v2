CREATE SCHEMA IF NOT EXISTS arepl;

---- Tables

CREATE TABLE arepl.last_gddl_update (LIKE aredl.last_gddl_update INCLUDING ALL);
CREATE TABLE arepl.levels (LIKE aredl.levels INCLUDING ALL);
CREATE TABLE arepl.levels_created (LIKE aredl.levels_created INCLUDING ALL);
CREATE TABLE arepl.packs (LIKE aredl.packs INCLUDING ALL);
CREATE TABLE arepl.pack_tiers (LIKE aredl.pack_tiers INCLUDING ALL);
CREATE TABLE arepl.pack_levels (LIKE aredl.pack_levels INCLUDING ALL);
CREATE TABLE arepl.position_history (LIKE aredl.position_history INCLUDING ALL);
CREATE TABLE arepl.records (LIKE aredl.records INCLUDING ALL);
CREATE TABLE arepl.shifts (LIKE aredl.shifts INCLUDING ALL);
CREATE TABLE arepl.recurrent_shifts (LIKE aredl.recurrent_shifts INCLUDING ALL);
CREATE TABLE arepl.submissions (LIKE aredl.submissions INCLUDING ALL);
CREATE TABLE arepl.submission_history (LIKE aredl.submission_history INCLUDING ALL);

ALTER TABLE arepl.records
  ADD COLUMN IF NOT EXISTS completion_time BIGINT NOT NULL DEFAULT 0;
ALTER TABLE arepl.submissions
  ADD COLUMN IF NOT EXISTS completion_time BIGINT NOT NULL DEFAULT 0;

CREATE SEQUENCE arepl.position_history_i_seq
  OWNED BY arepl.position_history.i;

ALTER TABLE arepl.position_history
  ALTER COLUMN i SET DEFAULT nextval('arepl.position_history_i_seq');

---- Triggers/functions

CREATE FUNCTION arepl.point_formula(pos int, level_count int) RETURNS int AS
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

CREATE FUNCTION arepl.level_place() RETURNS TRIGGER AS
$$
BEGIN
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
BEGIN
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
    move_dir int;
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
BEGIN
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
BEGIN
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


CREATE FUNCTION arepl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_record_time
BEFORE UPDATE ON arepl.records
FOR EACH ROW
EXECUTE FUNCTION arepl.update_record_time();

CREATE FUNCTION arepl.update_record_placement()
RETURNS TRIGGER AS $$
  BEGIN
    UPDATE arepl.records
    SET placement_order = sub.row_num - 1
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY level_id ORDER BY created_at) AS row_num
        FROM arepl.records
        WHERE EXISTS (
            SELECT 1 FROM new_table as n WHERE n.level_id = arepl.records.level_id
        )
    ) AS sub
    WHERE arepl.records.id = sub.id;
    RETURN null;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_record_placement_on_update
AFTER UPDATE ON arepl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION arepl.update_record_placement();

CREATE TRIGGER update_record_placement_on_insert
AFTER INSERT ON arepl.records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION arepl.update_record_placement();

---- Views

CREATE VIEW arepl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc FROM arepl.pack_levels pl GROUP BY pl.pack_id
    )
    SELECT r.submitted_by AS user_id, pl.pack_id
    FROM arepl.records r
    JOIN arepl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(r.*) = pcl.lc;

CREATE VIEW arepl.packs_points AS
    SELECT p.*, ROUND(SUM(l.points) * 0.5)::INTEGER AS points
    FROM arepl.packs p
    JOIN arepl.pack_levels pl ON p.id = pl.pack_id
    JOIN arepl.levels l ON l.id = pl.level_id
    GROUP BY p.id;

CREATE VIEW arepl.user_pack_points AS
    SELECT cp.user_id, SUM(p.points)::INTEGER AS points
    FROM arepl.completed_packs cp
    JOIN arepl.packs_points p ON p.id = cp.pack_id
    GROUP BY cp.user_id;

CREATE VIEW arepl.min_placement_country_records AS
WITH subquery AS (
    SELECT
        r.*,
        u.country,
        row_number() OVER (
          PARTITION BY r.level_id, u.country
          ORDER BY r.placement_order
        ) AS order_pos
    FROM arepl.records r
    JOIN users u ON u.id = r.submitted_by
)
SELECT *
FROM subquery
WHERE order_pos = 1;

CREATE VIEW arepl.min_placement_clans_records AS
    WITH subquery AS (
        SELECT
            r.*,
            cm.clan_id,
            row_number() over ( PARTITION BY r.level_id, cm.clan_id ORDER BY r.placement_order) as order_pos
        FROM arepl.records r
        JOIN clan_members cm ON cm.user_id = r.submitted_by
    )
    SELECT *
    FROM subquery
    WHERE order_pos = 1;

CREATE OR REPLACE VIEW arepl.submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM arepl.submissions;

---- Materialized Views

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

	DELETE FROM arepl.records ar1
	USING arepl.records ar2
	WHERE ar1.submitted_by = p_secondary_user
	AND ar1.level_id = ar2.level_id
	AND ar2.submitted_by = p_primary_user;

	DELETE FROM arepl.submissions as1
	USING arepl.submissions as2
	WHERE as1.submitted_by = p_secondary_user
	AND as1.level_id = as2.level_id
	AND as2.submitted_by = p_primary_user;

	DELETE FROM arepl.levels_created ac1
	USING arepl.levels_created ac2
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

	UPDATE arepl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE arepl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE arepl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE arepl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;

	INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
	SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
	FROM users WHERE id = p_secondary_user;

	UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

	DELETE FROM users WHERE id = p_secondary_user;

END;
$$ LANGUAGE plpgsql;
