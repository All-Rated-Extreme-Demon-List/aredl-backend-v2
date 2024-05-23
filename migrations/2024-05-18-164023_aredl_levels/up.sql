CREATE TABLE aredl_levels (
    id uuid DEFAULT uuid_generate_v4(),
    position INT NOT NULL,
    name VARCHAR NOT NULL,
    publisher_id uuid NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    points INT NOT NULL DEFAULT 0,
    legacy BOOLEAN NOT NULL DEFAULT false,
    level_id INT NOT NULL CHECK (level_id > 0),
    two_player BOOLEAN NOT NULL,
    PRIMARY KEY(id),
    UNIQUE (level_id, two_player)
);

CREATE TABLE aredl_position_history (
    i SERIAL,
    new_position INT,
    old_position INT,
    legacy BOOLEAN,
    affected_level uuid NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(i),
    CONSTRAINT fk_level
        FOREIGN KEY(affected_level)
        REFERENCES aredl_levels(id)
);

CREATE FUNCTION aredl_point_formula(pos int, level_count int) RETURNS int AS
$$
DECLARE
    a float;
    b float;
BEGIN
    IF level_count <= 1 THEN
        return 500;
    END IF;
    b := (level_count - 1) * 0.0005832492374192;
    a := 6000 * sqrt(b);
    return ROUND((a / sqrt((CAST(pos AS float) - 1) / 50 + b) - 1000));
END
$$ LANGUAGE plpgsql;

CREATE FUNCTION aredl_level_place() RETURNS TRIGGER AS
$$
BEGIN
    UPDATE aredl_levels
    SET position = position + 1
    WHERE position >= NEW.position AND id <> NEW.id;

    INSERT INTO aredl_position_history(new_position, old_position, legacy, affected_level)
    VALUES (NEW.position, NULL, NEW.legacy, NEW.id);

    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_level_place
AFTER INSERT ON "aredl_levels"
FOR EACH ROW
EXECUTE PROCEDURE aredl_level_place();

CREATE FUNCTION aredl_level_move() RETURNS TRIGGER AS
$$
DECLARE
    move_dir int;
    legacy_history boolean;
BEGIN
    IF NEW.position = OLD.position AND NEW.legacy = OLD.legacy THEN
        RETURN null;
    END IF;
    UPDATE aredl_levels
    SET position = position + (CASE WHEN NEW.position < OLD.position THEN 1 ELSE -1 END)
    WHERE id <> NEW.id AND position
        BETWEEN LEAST(NEW.position, OLD.position)
        AND GREATEST(NEW.position, OLD.position);

    legacy_history := NULL;
    IF NEW.legacy <> OLD.legacy THEN
        legacy_history := NEW.legacy;
    END IF;

    INSERT INTO aredl_position_history(new_position, old_position, legacy, affected_level)
    VALUES (NEW.position, OLD.position, legacy_history, NEW.id);
    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_level_move
AFTER UPDATE OF "position", "legacy" ON "aredl_levels"
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl_level_move();

CREATE FUNCTION aredl_max_list_pos() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM aredl_levels WHERE legacy = false;
$$ LANGUAGE sql;

CREATE FUNCTION aredl_max_list_pos_legacy() RETURNS int AS
$$
    SELECT COALESCE(max(position), 0) AS result FROM aredl_levels;
$$ LANGUAGE sql;

CREATE FUNCTION aredl_validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
	lowestPos INT;
	highestPos INT;
BEGIN
	IF NEW.legacy THEN
		highestPos := aredl_max_list_pos_legacy() + 1;
		lowestPos := aredl_max_list_pos() + 1;
	ELSE
		highestPos := aredl_max_list_pos() + 1;
		lowestPos := 1;
	END IF;
	IF NEW.position > highestPos OR NEW.position < lowestPos THEN
		RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
	END IF;
	RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_validate_position_insert
BEFORE INSERT ON "aredl_levels"
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl_validate_position_insert();

CREATE FUNCTION aredl_validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
	lowestPos INT;
	highestPos INT;
BEGIN
	IF NEW.legacy THEN
		IF NEW.legacy <> OLD.legacy THEN
            lowestPos := aredl_max_list_pos();
        ELSE
            lowestPos := aredl_max_list_pos() + 1;
        END IF;
        highestPos := aredl_max_list_pos_legacy();
	ELSE
	    IF NEW.legacy <> OLD.legacy THEN
		    highestPos := aredl_max_list_pos() + 1;
	    ELSE
	        highestPos := aredl_max_list_pos();
	    END IF;
        lowestPos := 1;
	END IF;
	IF NEW.position > highestPos OR NEW.position < lowestPos THEN
		RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
	END IF;
	RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_validate_position_update
BEFORE UPDATE OF "position", "legacy" ON "aredl_levels"
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl_validate_position_update();

CREATE VIEW aredl_position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, new_position, old_position, legacy, created_at, affected_level
    FROM aredl_position_history
),
full_history AS (
	SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) as prev_pos, legacy, legacy AS prev_legacy, created_at AS action_at, affected_level AS cause
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
		r.affected_level as cause
	FROM ranked_history r
	INNER JOIN full_history h ON r.i = h.i + 1
)
SELECT id as affected_level, position, legacy, action_at, cause
FROM full_history
WHERE prev_pos <> position OR prev_legacy <> legacy OR prev_pos IS NULL;

CREATE FUNCTION aredl_levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    new.points := aredl_point_formula(new.position, CAST((SELECT COUNT(*) FROM aredl_levels) AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_levels_points_before_update
BEFORE UPDATE OF "position" ON "aredl_levels"
FOR EACH ROW
EXECUTE PROCEDURE aredl_levels_points_before_update();

CREATE FUNCTION aredl_levels_points_before_insert() RETURNS TRIGGER AS
$$
BEGIN
    new.points := aredl_point_formula(new.position, CAST((SELECT COUNT(*) FROM aredl_levels) + 1 AS INT));
    RETURN new;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_levels_points_before_insert
BEFORE INSERT ON "aredl_levels"
FOR EACH ROW
EXECUTE PROCEDURE aredl_levels_points_before_insert();

CREATE FUNCTION aredl_levels_points_after_insert() RETURNS TRIGGER AS
$$
BEGIN
    UPDATE aredl_levels
    SET points = aredl_point_formula(position, CAST((SELECT COUNT(*) FROM aredl_levels) AS INT));
    RETURN null;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER aredl_levels_points_after_insert
AFTER INSERT ON "aredl_levels"
FOR EACH STATEMENT
EXECUTE PROCEDURE aredl_levels_points_after_insert();