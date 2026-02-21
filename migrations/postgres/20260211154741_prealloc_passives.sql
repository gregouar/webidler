CREATE TABLE
    characters_builds (
        build_id BIGSERIAL PRIMARY KEY,
        --
        character_id UUID NOT NULL,
        --
        title TEXT NOT NULL,
        passives_data BYTEA,
        data_version TEXT NOT NULL,
        --
        created_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT now (),
        deleted_at TIMESTAMPTZ DEFAULT NULL,
        --
        FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE CASCADE,
        UNIQUE (character_id, title)
    );

CREATE INDEX idx_characters_builds_character_id ON characters_builds (character_id);

CREATE INDEX idx_characters_builds_data_version ON characters_builds (data_version);