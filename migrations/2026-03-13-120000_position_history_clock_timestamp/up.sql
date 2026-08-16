ALTER TABLE aredl.position_history
    ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.position_history
    ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();
