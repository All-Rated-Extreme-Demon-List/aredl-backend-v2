CREATE OR REPLACE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid) RETURNS void AS
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

    DELETE FROM aredl.records ar1
	USING aredl.records ar2
	WHERE ar1.submitted_by = p_secondary_user
	AND ar1.level_id = ar2.level_id
	AND ar2.submitted_by = p_primary_user;

	DELETE FROM aredl.submissions as1
	USING aredl.submissions as2
	WHERE as1.submitted_by = p_secondary_user
	AND as1.level_id = as2.level_id
	AND as2.submitted_by = p_primary_user;

	DELETE FROM aredl.levels_created ac1
	USING aredl.levels_created ac2
	WHERE ac1.user_id = p_secondary_user
	AND ac1.level_id = ac2.level_id
	AND ac2.user_id = p_primary_user;


	DELETE FROM arepl.records ar1
	USING arepl.records ar2
	WHERE ar1.submitted_by = p_secondary_user
	AND ar1.level_id = ar2.level_id
	AND ar2.submitted_by = p_primary_user;

	DELETE FROM arepl.submissions as1
	USING arepl.submissions as2
	WHERE as1.submitted_by = p_secondary_user
	AND as1.level_id = as2.level_id
	AND as2.submitted_by = p_primary_user;

	DELETE FROM arepl.levels_created ac1
	USING arepl.levels_created ac2
	WHERE ac1.user_id = p_secondary_user
	AND ac1.level_id = ac2.level_id
	AND ac2.user_id = p_primary_user;


	DELETE FROM clan_members cm1
	USING clan_members cm2
	WHERE cm1.user_id = p_secondary_user
	AND cm2.user_id = p_primary_user;

	DELETE FROM user_roles ur1
	USING user_roles ur2
	WHERE ur1.user_id = p_secondary_user
	AND ur1.role_id = ur2.role_id
	AND ur2.user_id = p_primary_user;

    UPDATE aredl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE aredl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE aredl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;

	UPDATE arepl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE arepl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
	UPDATE arepl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE arepl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;

    UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
	UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;



	INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
	SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
	FROM users WHERE id = p_secondary_user;

	UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

	DELETE FROM users WHERE id = p_secondary_user;

END;
$$ LANGUAGE plpgsql;
