-- Market table
CREATE TABLE market (
    market_id INTEGER NOT NULL PRIMARY KEY,
    --
    character_id TEXT NOT NULL,
    private_sale TEXT,
    rejected BOOLEAN NOT NULL DEFAULT 0,
    --
    price REAL NOT NULL,
    --
    -- TODO: Replace later by json search? Might need to drop the sqlite compatibility
    base_item_id TEXT NOT NULL,
    item_name TEXT NOT NULL,
    item_rarity TEXT NOT NULL,
    item_level INT NOT NULL,
    item_armor REAL,
    item_block REAL,
    item_damages REAL,
    -- 
    item_data BLOB NOT NULL,
    --JSONB
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

CREATE INDEX idx_market_base_item_id ON market (base_item_id)
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
    market_id INTEGER NOT NULL,
    category TEXT NOT NULL,
    deleted_at TIMESTAMP,
    FOREIGN KEY (market_id) REFERENCES market(market_id) ON DELETE CASCADE
);

CREATE INDEX idx_market_categories_market_id ON market_categories (market_id);

CREATE INDEX idx_market_categories_category ON market_categories (category)
WHERE
    deleted_at IS NULL;

-- Item stats
CREATE TABLE market_stats (
    market_id INTEGER NOT NULL,
    item_stat TEXT NOT NULL,
    stat_value REAL NOT NULL,
    deleted_at TIMESTAMP,
    FOREIGN KEY (market_id) REFERENCES market(market_id) ON DELETE CASCADE
);

CREATE INDEX idx_market_stats_market_id ON market_stats (market_id);

CREATE INDEX idx_market_stats_item_stat ON market_stats (item_stat, stat_value)
WHERE
    deleted_at IS NULL;

-- Unique character name to find character by name
-- ALTER TABLE
--     characters
-- ADD
--     CONSTRAINT uc_characters_character_name UNIQUE (character_name);