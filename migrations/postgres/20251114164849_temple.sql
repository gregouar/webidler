ALTER TABLE
    characters
ADD
    COLUMN resource_gold DOUBLE PRECISION NOT NULL DEFAULT 0;

ALTER TABLE
    characters_data
ADD
    COLUMN benedictions_data BYTEA;