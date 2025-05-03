DROP INDEX IF EXISTS ix_aredl_recurrent_shifts_user_weekday;
DROP INDEX IF EXISTS ix_aredl_shifts_end;
DROP INDEX IF EXISTS ix_aredl_shifts_start;
DROP INDEX IF EXISTS ix_aredl_shifts_user_status;

DROP TABLE IF EXISTS aredl_recurrent_shifts;
DROP TABLE IF EXISTS aredl_shifts;

DROP TYPE IF EXISTS weekday;
DROP TYPE IF EXISTS shift_status;