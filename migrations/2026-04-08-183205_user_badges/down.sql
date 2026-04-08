ALTER TABLE users
    DROP COLUMN IF EXISTS featured_badge_code;

DROP TABLE IF EXISTS user_badges;
