-- replace legacy bool with a status field and backfill

CREATE TYPE level_status AS ENUM ('Pending', 'MainList', 'Legacy', 'Removed');

ALTER TABLE aredl.levels
    ADD COLUMN status level_status,
    ADD COLUMN requires_raw_footage BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE aredl.levels
SET status = CASE WHEN legacy THEN 'Legacy'::level_status ELSE 'MainList'::level_status END;

ALTER TABLE aredl.levels
    ALTER COLUMN status SET NOT NULL,
    ALTER COLUMN position DROP NOT NULL;

ALTER TABLE arepl.levels
    ADD COLUMN status level_status,
    ADD COLUMN requires_raw_footage BOOLEAN NOT NULL DEFAULT FALSE;

UPDATE arepl.levels
SET status = CASE WHEN legacy THEN 'Legacy'::level_status ELSE 'MainList'::level_status END;

ALTER TABLE arepl.levels
    ALTER COLUMN status SET NOT NULL,
    ALTER COLUMN position DROP NOT NULL;


ALTER TABLE aredl.position_history
    ADD COLUMN old_status level_status,
    ADD COLUMN new_status level_status;

ALTER TABLE arepl.position_history
    ADD COLUMN old_status level_status,
    ADD COLUMN new_status level_status;


-- backfill pos history with status changes based on the legacy flag
WITH states AS (
    SELECT
        ph.i AS history_id,
        ph.new_position,
        ph.legacy,
        LAG(ph.new_position) OVER (PARTITION BY ph.affected_level ORDER BY ph.i) AS prev_position,
        LAG(ph.legacy) OVER (PARTITION BY ph.affected_level ORDER BY ph.i) AS prev_legacy
    FROM aredl.position_history ph
)
UPDATE aredl.position_history ph
SET old_status = CASE
        WHEN ph.old_position IS NULL AND s.prev_position IS NULL THEN NULL -- no previous entry so it's the first placement, so no old status
        WHEN s.prev_legacy THEN 'Legacy'::level_status
        ELSE 'MainList'::level_status
    END,
    new_status = CASE
        WHEN s.legacy THEN 'Legacy'::level_status
        ELSE 'MainList'::level_status
    END
FROM states s
WHERE s.history_id = ph.i;

WITH states AS (
    SELECT
        ph.i AS history_id,
        ph.new_position,
        ph.legacy,
        LAG(ph.new_position) OVER (PARTITION BY ph.affected_level ORDER BY ph.i) AS prev_position,
        LAG(ph.legacy) OVER (PARTITION BY ph.affected_level ORDER BY ph.i) AS prev_legacy
    FROM arepl.position_history ph
)
UPDATE arepl.position_history ph
SET old_status = CASE
        WHEN ph.old_position IS NULL AND s.prev_position IS NULL THEN NULL
        WHEN s.prev_legacy THEN 'Legacy'::level_status
        ELSE 'MainList'::level_status
    END,
    new_status = CASE
        WHEN s.legacy THEN 'Legacy'::level_status
        ELSE 'MainList'::level_status
    END
FROM states s
WHERE s.history_id = ph.i;

ALTER TABLE aredl.position_history
    ALTER COLUMN new_status SET NOT NULL;

ALTER TABLE arepl.position_history
    ALTER COLUMN new_status SET NOT NULL;

-- drop old objects that depends on the legacy flag or on the position
DROP MATERIALIZED VIEW IF EXISTS aredl.position_history_full_view;
DROP MATERIALIZED VIEW IF EXISTS arepl.position_history_full_view;
DROP MATERIALIZED VIEW IF EXISTS aredl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.user_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.clans_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS arepl.country_leaderboard;
DROP MATERIALIZED VIEW IF EXISTS aredl.country_created_levels;
DROP MATERIALIZED VIEW IF EXISTS arepl.country_created_levels;
DROP MATERIALIZED VIEW IF EXISTS aredl.clans_created_levels;
DROP MATERIALIZED VIEW IF EXISTS arepl.clans_created_levels;

DROP TRIGGER IF EXISTS level_place ON aredl.levels;
DROP TRIGGER IF EXISTS level_place_history ON aredl.levels;
DROP TRIGGER IF EXISTS level_move ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON aredl.levels;
DROP TRIGGER IF EXISTS levels_points_before_insert ON aredl.levels;
DROP TRIGGER IF EXISTS levels_points_before_update ON aredl.levels;
DROP TRIGGER IF EXISTS levels_points_after_insert ON aredl.levels;

DROP TRIGGER IF EXISTS level_place ON arepl.levels;
DROP TRIGGER IF EXISTS level_place_history ON arepl.levels;
DROP TRIGGER IF EXISTS level_move ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON arepl.levels;
DROP TRIGGER IF EXISTS levels_points_before_insert ON arepl.levels;
DROP TRIGGER IF EXISTS levels_points_before_update ON arepl.levels;
DROP TRIGGER IF EXISTS levels_points_after_insert ON arepl.levels;

DROP FUNCTION IF EXISTS aredl.level_place() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_move() CASCADE;
DROP FUNCTION IF EXISTS aredl.max_list_pos() CASCADE;
DROP FUNCTION IF EXISTS aredl.max_list_pos_legacy() CASCADE;
DROP FUNCTION IF EXISTS aredl.validate_position_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.validate_position_update() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_before_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_before_update() CASCADE;
DROP FUNCTION IF EXISTS aredl.levels_points_after_insert() CASCADE;
DROP FUNCTION IF EXISTS aredl.recalculate_points() CASCADE;

DROP FUNCTION IF EXISTS arepl.level_place() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_move() CASCADE;
DROP FUNCTION IF EXISTS arepl.max_list_pos() CASCADE;
DROP FUNCTION IF EXISTS arepl.max_list_pos_legacy() CASCADE;
DROP FUNCTION IF EXISTS arepl.validate_position_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.validate_position_update() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_before_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_before_update() CASCADE;
DROP FUNCTION IF EXISTS arepl.levels_points_after_insert() CASCADE;
DROP FUNCTION IF EXISTS arepl.recalculate_points() CASCADE;

-- remove legacy
ALTER TABLE aredl.position_history
    DROP COLUMN legacy;

ALTER TABLE arepl.position_history
    DROP COLUMN legacy;

ALTER TABLE aredl.levels
    DROP COLUMN legacy;

ALTER TABLE arepl.levels
    DROP COLUMN legacy;

-- update other levels position when directly placing a level (not going through pending)
CREATE FUNCTION aredl.level_place() RETURNS TRIGGER AS
$$
BEGIN
    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        RETURN NULL;
    END IF;

    UPDATE aredl.levels
    SET position = position + 1
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND position >= NEW.position;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place
AFTER INSERT ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_place();

-- log history when adding levels 

CREATE FUNCTION aredl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above UUID;
    below UUID;
BEGIN
    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.level_place_history();

-- update other levels position when moving a level (either changing the position on the list, or moving in/out of pending/removed)

CREATE FUNCTION aredl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
BEGIN
    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE aredl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1 -- new placement, shift everything below it down by 1
        WHEN old_placed AND NOT new_placed THEN -1 -- removed from placement, shift everything below the old position up by 1
        WHEN OLD.position < NEW.position THEN -1 -- otherwise regular move like before, shift everything between the old and new position up by 1 if moving down
        ELSE 1 -- or down by 1 if moving up
    END
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND (
          (NOT old_placed AND new_placed AND position >= NEW.position)
          OR
          (old_placed AND NOT new_placed AND position > OLD.position)
          OR
          (old_placed AND new_placed AND position BETWEEN LEAST(NEW.position, OLD.position) AND GREATEST(NEW.position, OLD.position))
      );

    IF new_placed THEN
        above := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF position, status ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_move();

-- boundaries for list positions

CREATE FUNCTION aredl.max_list_pos() RETURNS INT AS
$$
    SELECT COALESCE(MAX(position), 0) FROM aredl.levels WHERE status = 'MainList';
$$ LANGUAGE sql;

CREATE FUNCTION aredl.max_list_pos_legacy() RETURNS INT AS
$$
    SELECT COALESCE(MAX(position), 0) FROM aredl.levels WHERE status IN ('MainList', 'Legacy');
$$ LANGUAGE sql;

-- ensures that when adding a level, unplaced statuses have no position, and placed ones have one withing bounds
CREATE FUNCTION aredl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        highestPos := aredl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := aredl.max_list_pos_legacy() + 1;
        lowestPos := aredl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_insert
BEFORE INSERT ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.validate_position_insert();

-- ensures that when updating a level, placed status have a position and unplaced status do not
-- validates position boundaries
-- also ensure that the raw required flag is set to false when a level is moved into a placed status (the top 400 check should take over)

CREATE FUNCTION aredl.validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF OLD.status NOT IN ('MainList', 'Legacy') THEN
        NEW.requires_raw_footage := FALSE;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        IF OLD.status = 'MainList' THEN
            highestPos := aredl.max_list_pos();
        ELSE
            highestPos := aredl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := aredl.max_list_pos();
        ELSE
            lowestPos := aredl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := aredl.max_list_pos_legacy();
        ELSE
            highestPos := aredl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_update
BEFORE UPDATE OF position, status ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.validate_position_update();

-- points calculation

CREATE FUNCTION aredl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN aredl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF position, status ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.levels_points_before_update();

CREATE FUNCTION aredl.levels_points_before_insert() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN aredl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') + 1 AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_insert
BEFORE INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.levels_points_before_insert();

-- recalculate points for main list levels
CREATE FUNCTION aredl.recalculate_points() RETURNS VOID AS
$$
BEGIN
    UPDATE aredl.levels
    SET points = CASE
        WHEN status = 'MainList' THEN aredl.point_formula(position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION aredl.levels_points_after_insert() RETURNS TRIGGER AS
$$
BEGIN
    PERFORM aredl.recalculate_points();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_after_insert
AFTER INSERT ON aredl.levels
FOR EACH STATEMENT
EXECUTE PROCEDURE aredl.levels_points_after_insert();

-- same for platformer


CREATE FUNCTION arepl.level_place() RETURNS TRIGGER AS
$$
BEGIN
    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        RETURN NULL;
    END IF;

    UPDATE arepl.levels
    SET position = position + 1
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND position >= NEW.position;

    RETURN NULL;
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
    above UUID;
    below UUID;
BEGIN
    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.level_place_history();

CREATE FUNCTION arepl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
BEGIN
    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE arepl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND (
          (NOT old_placed AND new_placed AND position >= NEW.position)
          OR
          (old_placed AND NOT new_placed AND position > OLD.position)
          OR
          (old_placed AND new_placed AND position BETWEEN LEAST(NEW.position, OLD.position) AND GREATEST(NEW.position, OLD.position))
      );

    IF new_placed THEN
        above := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF position, status ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.level_move();

CREATE FUNCTION arepl.max_list_pos() RETURNS INT AS
$$
    SELECT COALESCE(MAX(position), 0) FROM arepl.levels WHERE status = 'MainList';
$$ LANGUAGE sql;

CREATE FUNCTION arepl.max_list_pos_legacy() RETURNS INT AS
$$
    SELECT COALESCE(MAX(position), 0) FROM arepl.levels WHERE status IN ('MainList', 'Legacy');
$$ LANGUAGE sql;

CREATE FUNCTION arepl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        highestPos := arepl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := arepl.max_list_pos_legacy() + 1;
        lowestPos := arepl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
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
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF OLD.status NOT IN ('MainList', 'Legacy') THEN
        NEW.requires_raw_footage := FALSE;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        IF OLD.status = 'MainList' THEN
            highestPos := arepl.max_list_pos();
        ELSE
            highestPos := arepl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := arepl.max_list_pos();
        ELSE
            lowestPos := arepl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := arepl.max_list_pos_legacy();
        ELSE
            highestPos := arepl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER validate_position_update
BEFORE UPDATE OF position, status ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.validate_position_update();

CREATE FUNCTION arepl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN arepl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF position, status ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.levels_points_before_update();

CREATE FUNCTION arepl.levels_points_before_insert() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN arepl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') + 1 AS INT))
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_before_insert
BEFORE INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.levels_points_before_insert();

CREATE FUNCTION arepl.recalculate_points() RETURNS VOID AS
$$
BEGIN
    UPDATE arepl.levels
    SET points = CASE
        WHEN status = 'MainList' THEN arepl.point_formula(position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') AS INT))
        ELSE 0
    END;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION arepl.levels_points_after_insert() RETURNS TRIGGER AS
$$
BEGIN
    PERFORM arepl.recalculate_points();
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER levels_points_after_insert
AFTER INSERT ON arepl.levels
FOR EACH STATEMENT
EXECUTE PROCEDURE arepl.levels_points_after_insert();

-- status/position constraints

ALTER TABLE aredl.levels
    ADD CONSTRAINT aredl_levels_status_position_check CHECK (
        (status IN ('Pending', 'Removed') AND position IS NULL)
        OR
        (status IN ('MainList', 'Legacy') AND position IS NOT NULL)
    );

ALTER TABLE arepl.levels
    ADD CONSTRAINT arepl_levels_status_position_check CHECK (
        (status IN ('Pending', 'Removed') AND position IS NULL)
        OR
        (status IN ('MainList', 'Legacy') AND position IS NOT NULL)
    );

-- pos history full views

CREATE MATERIALIZED VIEW aredl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, old_position, new_position, old_status, new_status,
           COALESCE(old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
           new_status IN ('MainList', 'Legacy') AS new_placed, created_at, affected_level
    FROM aredl.position_history
),
full_history AS (
    SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) AS prev_pos,
           new_status AS status, CAST(NULL AS level_status) AS prev_status,
           created_at AS action_at, affected_level AS cause, false AS moved
    FROM ranked_history
    WHERE old_status IS NULL
    UNION
    SELECT
        r.i,
        h.id,
        CASE -- position of each level after this change
            WHEN r.affected_level = h.id THEN r.new_position -- current level is also the one being changed, so use the new pos
            WHEN h.status NOT IN ('MainList', 'Legacy') THEN h.position -- current level isn't on the list
            WHEN NOT r.old_placed AND r.new_placed THEN -- the level being changed was placed
                CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END 
            WHEN r.old_placed AND NOT r.new_placed THEN -- the level being changed was removed
                CASE WHEN h.position > r.old_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position < r.new_position THEN -- the level being changed was moved down
                CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position > r.new_position THEN -- the level being changed was moved up
                CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
            ELSE h.position -- shouldn't happen
        END AS position,
        h.position AS prev_pos,
        CASE WHEN r.affected_level = h.id THEN r.new_status ELSE h.status END AS status,
        h.status AS prev_status,
        r.created_at AS action_at,
        r.affected_level AS cause,
        (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
    FROM ranked_history r
    INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS ( -- extract only entries that actually changed position/status or are the first entry for a level (prev_status IS NULL)
    SELECT i::INTEGER AS ord, id AS affected_level, position, moved, status, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff FROM filtered;

CREATE MATERIALIZED VIEW arepl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, old_position, new_position, old_status, new_status,
           COALESCE(old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
           new_status IN ('MainList', 'Legacy') AS new_placed, created_at, affected_level
    FROM arepl.position_history
),
full_history AS (
    SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) AS prev_pos,
           new_status AS status, CAST(NULL AS level_status) AS prev_status,
           created_at AS action_at, affected_level AS cause, false AS moved
    FROM ranked_history
    WHERE old_status IS NULL
    UNION
    SELECT
        r.i,
        h.id,
        CASE
            WHEN r.affected_level = h.id THEN r.new_position
            WHEN h.status NOT IN ('MainList', 'Legacy') THEN h.position
            WHEN NOT r.old_placed AND r.new_placed THEN
                CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
            WHEN r.old_placed AND NOT r.new_placed THEN
                CASE WHEN h.position > r.old_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position < r.new_position THEN
                CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position > r.new_position THEN
                CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
            ELSE h.position
        END AS position,
        h.position AS prev_pos,
        CASE WHEN r.affected_level = h.id THEN r.new_status ELSE h.status END AS status,
        h.status AS prev_status,
        r.created_at AS action_at,
        r.affected_level AS cause,
        (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
    FROM ranked_history r
    INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS (
    SELECT i::INTEGER AS ord, id AS affected_level, position, moved, status, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff FROM filtered;

-- user leaderboards only used mainlist levels
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
    WHERE l.status = 'MainList'
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
    WHERE l.status = 'MainList'
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
      AND l.status = 'MainList'
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

CREATE MATERIALIZED VIEW arepl.clans_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT cm.clan_id, r.level_id
    FROM arepl.records r
    JOIN clan_members cm ON r.submitted_by = cm.user_id
    JOIN users u ON r.submitted_by = u.id
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND l.status = 'MainList'
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

CREATE MATERIALIZED VIEW aredl.country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT u.country, r.level_id
    FROM aredl.records r
    JOIN users u ON r.submitted_by = u.id
    JOIN aredl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND u.country IS NOT NULL
      AND u.country <> 0
      AND l.status = 'MainList'
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

CREATE MATERIALIZED VIEW arepl.country_leaderboard AS
WITH completed_levels AS (
    SELECT DISTINCT u.country, r.level_id
    FROM arepl.records r
    JOIN users u ON r.submitted_by = u.id
    JOIN arepl.levels l ON r.level_id = l.id
    WHERE u.ban_level = 0
      AND u.country IS NOT NULL
      AND u.country <> 0
      AND l.status = 'MainList'
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

-- creators list exclude removed levels
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
      AND l.status <> 'Removed'
),
levels_without_explicit_creators AS (
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
      AND l.status <> 'Removed'
)
SELECT country, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT country, level_id, creator_id, order_pos
FROM levels_without_explicit_creators;

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
    WHERE l.status <> 'Removed'
),
levels_without_explicit_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM aredl.levels l
    JOIN clan_members cm ON cm.user_id = l.publisher_id
    LEFT JOIN aredl.levels_created lc ON lc.level_id = l.id
    WHERE lc.level_id IS NULL
      AND l.status <> 'Removed'
)
SELECT clan_id, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT clan_id, level_id, creator_id, order_pos
FROM levels_without_explicit_creators;

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
      AND l.status <> 'Removed'
),
levels_without_explicit_creators AS (
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
      AND l.status <> 'Removed'
)
SELECT country, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT country, level_id, creator_id, order_pos
FROM levels_without_explicit_creators;

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
    WHERE l.status <> 'Removed'
),
levels_without_explicit_creators AS (
    SELECT
        cm.clan_id,
        l.id AS level_id,
        l.publisher_id AS creator_id,
        l.position AS order_pos
    FROM arepl.levels l
    JOIN clan_members cm ON cm.user_id = l.publisher_id
    LEFT JOIN arepl.levels_created lc ON lc.level_id = l.id
    WHERE lc.level_id IS NULL
      AND l.status <> 'Removed'
)
SELECT clan_id, level_id, creator_id, order_pos
FROM explicit_creators
UNION
SELECT clan_id, level_id, creator_id, order_pos
FROM levels_without_explicit_creators;

CREATE INDEX arepl_clans_created_levels_clan_idx
    ON arepl.clans_created_levels (clan_id, order_pos, level_id, creator_id);

-- backfill points
SELECT aredl.recalculate_points();
SELECT arepl.recalculate_points();
