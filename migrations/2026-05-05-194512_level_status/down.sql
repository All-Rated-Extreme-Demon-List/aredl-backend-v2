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
DROP VIEW IF EXISTS arepl.user_pack_points;
DROP VIEW IF EXISTS aredl.user_pack_points;
DROP VIEW IF EXISTS arepl.packs_points;
DROP VIEW IF EXISTS aredl.packs_points;
DROP VIEW IF EXISTS arepl.completed_packs;
DROP VIEW IF EXISTS aredl.completed_packs;

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
    WHEN new_status::TEXT IN ('MainList', 'Published') THEN FALSE
    ELSE NULL
END;

ALTER TABLE aredl.position_history
    ADD COLUMN legacy BOOLEAN;

UPDATE aredl.position_history
SET legacy = CASE
    WHEN new_status = 'Legacy' THEN TRUE
    WHEN new_status::TEXT IN ('MainList', 'Published') THEN FALSE
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

DROP TYPE IF EXISTS level_status;
