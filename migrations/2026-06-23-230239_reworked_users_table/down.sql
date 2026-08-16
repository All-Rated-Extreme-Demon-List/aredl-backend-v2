ALTER TABLE users
    ADD COLUMN discord_banner TEXT,
    ADD COLUMN discord_accent_color INTEGER,
    ALTER COLUMN background_level SET DEFAULT 0;

UPDATE users
SET background_level = 0
WHERE background_level IS NULL;

ALTER TABLE users
    ALTER COLUMN background_level SET NOT NULL;