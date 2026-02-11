CREATE TABLE
    characters_builds (
        build_id INTEGER NOT NULL PRIMARY KEY,
        --
        character_id TEXT NOT NULL,
        --
        title TEXT NOT NULL,
        passives_data BLOB,
        data_version TEXT NOT NULL,
        --
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMP,
        --
        FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE CASCADE,
        UNIQUE (character_id, title)
    );

CREATE INDEX idx_characters_builds_character_id ON characters_builds (character_id);

CREATE INDEX idx_characters_builds_data_version ON characters_builds (data_version);

-- CREATE UNIQUE INDEX idx_characters_builds_title ON characters_builds (character_id, title);