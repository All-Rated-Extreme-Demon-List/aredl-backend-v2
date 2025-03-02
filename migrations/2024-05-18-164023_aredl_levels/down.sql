DROP TRIGGER IF EXISTS aredl_levels_points_after_insert ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_levels_points_after_insert;

DROP TRIGGER IF EXISTS aredl_levels_points_before_insert ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_levels_points_before_insert;

DROP TRIGGER IF EXISTS aredl_levels_points_before_update ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_levels_points_before_update;

DROP MATERIALIZED VIEW IF EXISTS aredl_position_history_full_view;

DROP TRIGGER IF EXISTS aredl_validate_position_update ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_validate_position_update;

DROP FUNCTION IF EXISTS aredl_recalculate_points;

DROP TRIGGER IF EXISTS aredl_validate_position_insert ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_validate_position_insert;

DROP FUNCTION IF EXISTS aredl_max_list_pos_legacy;
DROP FUNCTION IF EXISTS aredl_max_list_pos;

DROP TRIGGER IF EXISTS aredl_level_move ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_level_move;

DROP TRIGGER IF EXISTS aredl_level_place_history ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_level_place_history;

DROP TRIGGER IF EXISTS aredl_level_place ON aredl_levels;
DROP FUNCTION IF EXISTS aredl_level_place;

DROP FUNCTION IF EXISTS aredl_point_formula;

DROP TABLE IF EXISTS aredl_position_history;

DROP TABLE IF EXISTS aredl_levels_created;

DROP TABLE IF EXISTS aredl_last_gddl_update;

DROP TABLE IF EXISTS aredl_levels;