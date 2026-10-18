CREATE TYPE clan_notifications AS ENUM ('None', 'NewOnly', 'All');

CREATE TABLE user_settings (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    clan_notifications clan_notifications NOT NULL DEFAULT 'None'
);

INSERT INTO user_settings (user_id) SELECT id FROM users;

CREATE FUNCTION create_user_settings() RETURNS trigger
LANGUAGE plpgsql AS $$
BEGIN
    INSERT INTO user_settings (user_id) VALUES (NEW.id);
    RETURN NEW;
END;
$$;

CREATE TRIGGER create_user_settings
AFTER INSERT ON users
FOR EACH ROW EXECUTE FUNCTION create_user_settings();
