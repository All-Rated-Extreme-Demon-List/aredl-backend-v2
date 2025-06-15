DROP TRIGGER IF EXISTS level_place ON aredl.levels;
DROP TRIGGER IF EXISTS level_place_history ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON aredl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON aredl.levels;
DROP FUNCTION IF EXISTS aredl.level_place();
DROP FUNCTION IF EXISTS aredl.level_place_history();
DROP FUNCTION IF EXISTS aredl.validate_position_insert();
DROP FUNCTION IF EXISTS aredl.validate_position_update();

DROP TRIGGER IF EXISTS level_place ON arepl.levels;
DROP TRIGGER IF EXISTS level_place_history ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_insert ON arepl.levels;
DROP TRIGGER IF EXISTS validate_position_update ON arepl.levels;
DROP FUNCTION IF EXISTS arepl.level_place();
DROP FUNCTION IF EXISTS arepl.level_place_history();
DROP FUNCTION IF EXISTS arepl.validate_position_insert();
DROP FUNCTION IF EXISTS arepl.validate_position_update();

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