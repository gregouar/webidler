CREATE TABLE market (
    item_id INTEGER NOT NULL PRIMARY KEY,
    --
    character_id TEXT NOT NULL,
    -- 'private_sale' means only one player can see it
    private_sale TEXT,
    -- 'rejected' means the private_sale target refused
    rejected BIT,
    price FLOAT NOT NULL,
    --
    -- TODO: Replace later by json search? Might need to drop the sqlite compatibility
    item_name TEXT NOT NULL,
    item_rarity TEXT NOT NULL,
    item_slot TEXT NOT NULL,
    item_categories TEXT NOT NULL,
    item_level INT NOT NULL,
    item_armor FLOAT,
    item_block FLOAT,
    item_damages FLOAT,
    --
    item_data BLOB NOT NULL,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP,
    --
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE,
    FOREIGN KEY(private_sale) REFERENCES characters(character_id) ON DELETE
    SET
        NULL
);

CREATE INDEX idx_market_character_id ON market (character_id, deleted_at);

CREATE INDEX idx_market_private_sale ON market (private_sale, deleted_at);

CREATE INDEX idx_market_deleted ON market (deleted_at);

-- Unique character name to find character by name

ALTER TABLE
    characters
ADD
    CONSTRAINT uc_characters_character_name UNIQUE (character_name);