
ALTER TABLE
    users DROP CONSTRAINT users_username_key;

CREATE UNIQUE INDEX uc_users_username_ci ON users (LOWER(username))
WHERE
    deleted_at IS NULL;
    

ALTER TABLE
    characters DROP CONSTRAINT uc_characters_character_name;

CREATE UNIQUE INDEX uc_characters_character_name_ci ON characters (LOWER(character_name))
WHERE
    deleted_at IS NULL;

    