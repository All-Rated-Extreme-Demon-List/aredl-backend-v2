DROP TRIGGER aredl_levels_points_after_insert ON aredl_levels;
DROP FUNCTION aredl_levels_points_after_insert;

DROP TRIGGER aredl_levels_points_before_insert ON aredl_levels;
DROP FUNCTION aredl_levels_points_before_insert;

DROP TRIGGER aredl_levels_points_before_update ON aredl_levels;
DROP FUNCTION aredl_levels_points_before_update;

DROP MATERIALIZED VIEW IF EXISTS aredl_position_history_full_view;

DROP TRIGGER aredl_validate_position_update ON aredl_levels;
DROP FUNCTION aredl_validate_position_update;

DROP FUNCTION aredl_recalculate_points;

DROP TRIGGER aredl_validate_position_insert ON aredl_levels;
DROP FUNCTION aredl_validate_position_insert;

DROP FUNCTION aredl_max_list_pos_legacy;
DROP FUNCTION aredl_max_list_pos;

DROP TRIGGER aredl_level_move ON aredl_levels;
DROP FUNCTION aredl_level_move;

DROP TRIGGER aredl_level_place_history ON aredl_levels;
DROP FUNCTION aredl_level_place_history;

DROP TRIGGER aredl_level_place ON aredl_levels;
DROP FUNCTION aredl_level_place;

DROP FUNCTION aredl_point_formula;

DROP TABLE aredl_position_history;

DROP TABLE aredl_levels_created;

DROP TABLE aredl_levels;