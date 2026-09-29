CREATE OR REPLACE FUNCTION prevent_role_inheritance_cycle()
RETURNS TRIGGER AS $$
DECLARE
    has_cycle BOOLEAN;
BEGIN
    IF NEW.inherits_from_role_id IS NULL THEN
        RETURN NEW;
    END IF;

    WITH RECURSIVE inherited_roles(id) AS (
        SELECT NEW.inherits_from_role_id
        UNION
        SELECT roles.inherits_from_role_id
        FROM roles
        INNER JOIN inherited_roles ON roles.id = inherited_roles.id
        WHERE roles.inherits_from_role_id IS NOT NULL
    )
    SELECT EXISTS (
        SELECT 1
        FROM inherited_roles
        WHERE id = NEW.id
    )
    INTO has_cycle;

    IF has_cycle THEN
        RAISE EXCEPTION 'Role inheritance cycle detected for role %', NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aredl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        highestPos := aredl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := aredl.max_list_pos_legacy() + 1;
        lowestPos := aredl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aredl.validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF OLD.status NOT IN ('MainList', 'Legacy') THEN
        NEW.requires_raw_footage := FALSE;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        IF OLD.status = 'MainList' THEN
            highestPos := aredl.max_list_pos();
        ELSE
            highestPos := aredl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := aredl.max_list_pos();
        ELSE
            lowestPos := aredl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := aredl.max_list_pos_legacy();
        ELSE
            highestPos := aredl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.validate_position_insert() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        highestPos := arepl.max_list_pos() + 1;
        lowestPos := 1;
    ELSE
        highestPos := arepl.max_list_pos_legacy() + 1;
        lowestPos := arepl.max_list_pos() + 1;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.validate_position_update() RETURNS TRIGGER AS
$$
DECLARE
    lowestPos INT;
    highestPos INT;
BEGIN
    IF NEW.status IS NULL THEN
        NEW.status := 'Pending';
    END IF;

    IF NEW.status NOT IN ('MainList', 'Legacy') THEN
        NEW.position := NULL;
        RETURN NEW;
    END IF;

    IF OLD.status NOT IN ('MainList', 'Legacy') THEN
        NEW.requires_raw_footage := FALSE;
    END IF;

    IF NEW.position IS NULL THEN
        RAISE EXCEPTION 'Position is required for status %', NEW.status;
    END IF;

    IF NEW.status = 'MainList' THEN
        IF OLD.status = 'MainList' THEN
            highestPos := arepl.max_list_pos();
        ELSE
            highestPos := arepl.max_list_pos() + 1;
        END IF;
        lowestPos := 1;
    ELSE
        IF OLD.status = 'MainList' THEN
            lowestPos := arepl.max_list_pos();
        ELSE
            lowestPos := arepl.max_list_pos() + 1;
        END IF;

        IF OLD.status IN ('MainList', 'Legacy') THEN
            highestPos := arepl.max_list_pos_legacy();
        ELSE
            highestPos := arepl.max_list_pos_legacy() + 1;
        END IF;
    END IF;
    IF NEW.position > highestPos OR NEW.position < lowestPos THEN
        RAISE EXCEPTION 'Position % outside of range % to %', NEW.position, lowestPos, highestPos;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
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

  PERFORM set_config('session_replication_role', 'replica', true);

  ----- deduplicate conflicting submissions (a submission for the same level exists for both users) -----
    WITH pairs AS (
        -- if secondary has an accepted one but not primary, keep secondary. in other cases keep primary.
        SELECT
            primary_s.level_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN secondary_s.id
                ELSE primary_s.id
            END AS keep_submission_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN primary_s.id
                ELSE secondary_s.id
            END AS discard_submission_id
        FROM aredl.submissions primary_s
        JOIN aredl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE aredl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM aredl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM aredl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- same for arepl
    WITH pairs AS (
        SELECT
            primary_s.level_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN secondary_s.id
                ELSE primary_s.id
            END AS keep_submission_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN primary_s.id
                ELSE secondary_s.id
            END AS discard_submission_id
        FROM arepl.submissions primary_s
        JOIN arepl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE arepl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM arepl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM arepl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- other deduplication

    DELETE FROM aredl.levels_created ac1
    USING aredl.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.level_id = ac2.level_id
        AND ac2.user_id = p_primary_user;

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

    DELETE FROM user_badges ub1
    USING user_badges ub2
    WHERE ub1.user_id = p_secondary_user
        AND ub1.badge_code = ub2.badge_code
        AND ub2.user_id = p_primary_user;

	DELETE FROM aredl.bounty_completed bc1
	USING aredl.bounty_completed bc2
	WHERE bc1.user_id = p_secondary_user
		AND bc1.bounty_id = bc2.bounty_id
		AND bc2.user_id = p_primary_user;

	DELETE FROM arepl.bounty_completed bc1
	USING arepl.bounty_completed bc2
	WHERE bc1.user_id = p_secondary_user
		AND bc1.bounty_id = bc2.bounty_id
		AND bc2.user_id = p_primary_user;

  ----- change ownership

    UPDATE aredl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE aredl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE aredl.bounty_completed SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE arepl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE arepl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;
	UPDATE arepl.bounty_completed SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_badges SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE users
    SET featured_badge_code = secondary_user.featured_badge_code
    FROM users AS secondary_user
    WHERE users.id = p_primary_user
      AND secondary_user.id = p_secondary_user
      AND users.featured_badge_code IS NULL
      AND secondary_user.featured_badge_code IS NOT NULL;

    PERFORM set_config('session_replication_role', 'origin', true);

  ----- log and delete

    INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
    SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
    FROM users WHERE id = p_secondary_user;

    UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

    DELETE FROM users WHERE id = p_secondary_user;
END;
$$;
