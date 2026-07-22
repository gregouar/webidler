ALTER TABLE
    characters
ADD
    COLUMN stamina DOUBLE PRECISION NOT NULL DEFAULT 0;

ALTER TABLE
    characters_data
ADD
    COLUMN skill_masteries_data BYTEA;