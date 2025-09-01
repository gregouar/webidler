-- Market table
CREATE TABLE market (
    item_id INTEGER NOT NULL PRIMARY KEY,
    --
    character_id TEXT NOT NULL,
    -- 'private_sale' means only one player can see it
    private_sale TEXT,
    -- 'rejected' means the private_sale target refused
    rejected BOOLEAN,
    price FLOAT NOT NULL,
    --
    -- TODO: Replace later by json search? Might need to drop the sqlite compatibility
    item_name TEXT NOT NULL,
    item_rarity TEXT NOT NULL,
    item_level INT NOT NULL,
    item_armor FLOAT,
    item_block FLOAT,
    item_damages FLOAT,
    --
    item_data JSONB NOT NULL,
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

CREATE INDEX idx_market_price ON market (price)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_name ON market (item_name)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_rarity ON market (item_rarity)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_level ON market (item_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_armor ON market (item_armor)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_block ON market (item_block)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damages ON market (item_damages)
WHERE
    deleted_at IS NULL;

-- Item categories
CREATE TABLE market_categories (
    item_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    FOREIGN KEY (item_id) REFERENCES market(item_id) ON DELETE CASCADE
);

CREATE INDEX idx_market_categories_category ON market_categories (category);

-- Unique character name to find character by name
-- ALTER TABLE
--     characters
-- ADD
--     CONSTRAINT uc_characters_character_name UNIQUE (character_name);