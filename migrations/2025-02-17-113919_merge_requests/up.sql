CREATE TABLE merge_requests (
    id uuid DEFAULT uuid_generate_v4(),
    primary_user uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secondary_user uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE, 
	is_rejected BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(id), 
    UNIQUE (primary_user),
    CHECK (primary_user <> secondary_user)
);

CREATE TABLE merge_logs ( 
    id uuid DEFAULT uuid_generate_v4(),
    primary_user uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    secondary_user uuid NOT NULL, 
    secondary_username VARCHAR NOT NULL,
    secondary_discord_id VARCHAR,
    secondary_global_name VARCHAR NOT NULL, 
    merged_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(id)
);

CREATE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid) RETURNS void AS
$$
BEGIN
	IF p_primary_user = p_secondary_user THEN
		RAISE EXCEPTION 'Cannot merge a user with themselves';
	END IF;

    IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_primary_user) THEN
        RAISE EXCEPTION 'Primary user % does not exist', p_primary_user;
    END IF;

    IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_secondary_user) THEN
        RAISE EXCEPTION 'Secondary user % does not exist', p_secondary_user;
    END IF;

	DELETE FROM aredl_records ar1
	USING aredl_records ar2
	WHERE ar1.submitted_by = p_secondary_user
	AND ar1.level_id = ar2.level_id
	AND ar2.submitted_by = p_primary_user;

	DELETE FROM aredl_submissions as1
	USING aredl_submissions as2
	WHERE as1.submitted_by = p_secondary_user
	AND as1.level_id = as2.level_id
	AND as2.submitted_by = p_primary_user;

	DELETE FROM aredl_levels_created ac1
	USING aredl_levels_created ac2
	WHERE ac1.user_id = p_secondary_user
	AND ac1.level_id = ac2.level_id
	AND ac2.user_id = p_primary_user;

	DELETE FROM clan_members cm1
	USING clan_members cm2
	WHERE cm1.user_id = p_secondary_user
	AND cm1.level_id = cm2.level_id
	AND cm2.user_id = p_primary_user;

	UPDATE aredl_records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl_submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl_levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE aredl_levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;

	INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
	SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
	FROM users WHERE id = p_secondary_user;

	UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

	DELETE FROM users WHERE id = p_secondary_user;

END;
$$ LANGUAGE plpgsql;