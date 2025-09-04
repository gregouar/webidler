ALTER TABLE
    characters DROP CONSTRAINT uc_characters_character_name;

ALTER TABLE
    characters
ADD
    CONSTRAINT uc_characters_character_name UNIQUE (character_name, deleted_at);