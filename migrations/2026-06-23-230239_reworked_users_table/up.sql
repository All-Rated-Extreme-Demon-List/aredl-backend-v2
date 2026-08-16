ALTER TABLE users
    DROP COLUMN discord_banner,
    DROP COLUMN discord_accent_color,
    ALTER COLUMN background_level DROP NOT NULL,
    ALTER COLUMN background_level DROP DEFAULT;