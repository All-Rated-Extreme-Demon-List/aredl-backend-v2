ALTER TABLE clan_members
    ADD CONSTRAINT clan_members_clan_id_user_id_key UNIQUE (clan_id, user_id),
    DROP CONSTRAINT clan_members_user_id_key;
