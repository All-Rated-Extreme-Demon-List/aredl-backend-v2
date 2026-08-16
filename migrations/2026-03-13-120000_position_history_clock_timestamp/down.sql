ALTER TABLE aredl.position_history
    ALTER COLUMN created_at SET DEFAULT NOW();

ALTER TABLE arepl.position_history
    ALTER COLUMN created_at SET DEFAULT NOW();
