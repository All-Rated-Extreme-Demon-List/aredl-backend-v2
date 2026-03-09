DROP TRIGGER IF EXISTS levels_points_before_update ON aredl.levels;
CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF "position" ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.levels_points_before_update();

DROP TRIGGER IF EXISTS levels_points_before_update ON arepl.levels;
CREATE TRIGGER levels_points_before_update
BEFORE UPDATE OF "position" ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.levels_points_before_update();
