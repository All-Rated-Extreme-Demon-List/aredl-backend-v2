CREATE OR REPLACE FUNCTION aredl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN aredl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM aredl.levels WHERE status = 'MainList') AS INT) + CASE WHEN OLD.status = 'MainList' THEN 0 ELSE 1 END)
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.levels_points_before_update() RETURNS TRIGGER AS
$$
BEGIN
    NEW.points := CASE
        WHEN NEW.status = 'MainList' THEN arepl.point_formula(NEW.position, CAST((SELECT COUNT(*) FROM arepl.levels WHERE status = 'MainList') AS INT) + CASE WHEN OLD.status = 'MainList' THEN 0 ELSE 1 END)
        ELSE 0
    END;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
