ALTER TABLE users
ADD COLUMN last_discord_avatar_update TIMESTAMP NULL DEFAULT NULL;

CREATE INDEX  IF NOT EXISTS idx_users_avatar_refresh
ON users (last_discord_avatar_update NULLS FIRST) WHERE discord_id IS NOT NULL;