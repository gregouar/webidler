-- Market table
CREATE TABLE market (
    market_id BIGINT NOT NULL PRIMARY KEY,
    --
    character_id UUID NOT NULL,
    recipient_id UUID,
    rejected BOOLEAN NOT NULL DEFAULT FALSE,
    --
    price DOUBLE PRECISION NOT NULL,
    --
    base_item_id TEXT NOT NULL,
    item_name TEXT NOT NULL,
    item_rarity TEXT NOT NULL,
    item_level INTEGER NOT NULL,
    item_armor DOUBLE PRECISION,
    item_block DOUBLE PRECISION,
    item_damages DOUBLE PRECISION,
    -- 
    item_data JSONB NOT NULL,
    --
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMPTZ,
    --
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE,
    FOREIGN KEY(recipient_id) REFERENCES characters(character_id) ON DELETE
    SET
        NULL
);

CREATE INDEX idx_market_character_id ON market (character_id, deleted_at);

CREATE INDEX idx_market_recipient_id ON market (recipient_id, deleted_at);

CREATE INDEX idx_market_deleted ON market (deleted_at);

CREATE INDEX idx_market_base_item_id ON market (base_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_name ON market (item_name)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_main_filters ON market (price, item_rarity, item_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_extra_filters ON market (item_armor, item_block, item_damages)
WHERE
    deleted_at IS NULL;

-- Item categories
CREATE TABLE market_categories (
    market_id BIGINT NOT NULL,
    category TEXT NOT NULL,
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (market_id) REFERENCES market(market_id) ON DELETE CASCADE
);

CREATE INDEX idx_market_categories_market_id ON market_categories (market_id);

CREATE INDEX idx_market_categories_category ON market_categories (category)
WHERE
    deleted_at IS NULL;

-- Item stats
CREATE TABLE market_stats (
    market_id BIGINT NOT NULL,
    item_stat JSONB NOT NULL,
    stat_modifier TEXT NOT NULL,
    stat_value DOUBLE PRECISION NOT NULL,
    deleted_at TIMESTAMPTZ,
    FOREIGN KEY (market_id) REFERENCES market(market_id) ON DELETE CASCADE
);

CREATE INDEX idx_market_stats_market_id ON market_stats (market_id);

CREATE INDEX idx_market_stats_item_stat ON market_stats (item_stat, stat_modifier, stat_value)
WHERE
    deleted_at IS NULL;

-- Unique character name to find character by name
ALTER TABLE
    characters
ADD
    CONSTRAINT uc_characters_character_name UNIQUE (character_name);