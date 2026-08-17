CREATE EXTENSION IF NOT EXISTS pg_trgm;

CREATE INDEX IF NOT EXISTS aredl_position_history_full_view_affected_ord_idx
ON aredl.position_history_full_view (affected_level, ord DESC);

CREATE INDEX IF NOT EXISTS aredl_levels_position_idx
ON aredl.levels (position)
WHERE position IS NOT NULL;

CREATE INDEX IF NOT EXISTS arepl_levels_position_idx
ON arepl.levels (position)
WHERE position IS NOT NULL;

CREATE INDEX IF NOT EXISTS users_global_name_trgm_idx
ON users USING gin (global_name gin_trgm_ops);

CREATE UNIQUE INDEX IF NOT EXISTS aredl_user_leaderboard_user_id_idx
ON aredl.user_leaderboard (user_id);

CREATE UNIQUE INDEX IF NOT EXISTS arepl_user_leaderboard_user_id_idx
ON arepl.user_leaderboard (user_id);

CREATE INDEX IF NOT EXISTS aredl_user_leaderboard_rank_user_idx
ON aredl.user_leaderboard (rank, user_id);

CREATE INDEX IF NOT EXISTS arepl_user_leaderboard_rank_user_idx
ON arepl.user_leaderboard (rank, user_id);

CREATE INDEX IF NOT EXISTS aredl_submissions_status_idx
ON aredl.submissions (status);

CREATE INDEX IF NOT EXISTS arepl_submissions_status_idx
ON arepl.submissions (status);
