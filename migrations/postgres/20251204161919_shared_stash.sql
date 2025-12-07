-- Keep old market table
DROP TABLE market_categories;

DROP TABLE market_stats;

ALTER TABLE market
RENAME TO market_old;

DROP INDEX idx_market_recipient_id;

DROP INDEX idx_market_deleted;

DROP INDEX idx_market_base_item_id;

DROP INDEX idx_market_name;

-- Stashes Table
CREATE TABLE
    stashes (
        stash_id UUID NOT NULL PRIMARY KEY,
        --
        user_id UUID NOT NULL,
        stash_type JSONB NOT NULL,
        --
        title TEXT,
        resource_gems DOUBLE PRECISION NOT NULL DEFAULT 0,
        max_items BIGINT NOT NULL,
        --
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMPTZ,
        --
        FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
    );

CREATE INDEX idx_stashes_user_id ON stashes (user_id);

CREATE INDEX idx_stashes_type ON stashes (stash_type);

-- Stash Items Table
CREATE TABLE
    stash_items (
        stash_item_id BIGSERIAL NOT NULL PRIMARY KEY,
        --
        stash_id UUID NOT NULL,
        character_id UUID,
        --
        base_item_id TEXT NOT NULL,
        item_name TEXT NOT NULL,
        item_rarity TEXT NOT NULL,
        item_level INTEGER NOT NULL,
        item_armor DOUBLE PRECISION,
        item_block DOUBLE PRECISION,
        item_damages DOUBLE PRECISION,
        item_damage_physical DOUBLE PRECISION,
        item_damage_fire DOUBLE PRECISION,
        item_damage_poison DOUBLE PRECISION,
        item_damage_storm DOUBLE PRECISION,
        item_crit_chance DOUBLE PRECISION,
        item_crit_damage DOUBLE PRECISION,
        -- 
        item_data JSONB NOT NULL,
        data_version TEXT NOT NULL,
        --
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMPTZ,
        --
        FOREIGN KEY (stash_id) REFERENCES stashes (stash_id) ON DELETE CASCADE,
        FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE SET NULL
    );

CREATE INDEX idx_stash_items_stash_id ON stash_items (stash_id, deleted_at);

CREATE INDEX idx_stash_items_name ON stash_items (stash_id, UPPER(item_name))
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_stash_items_created_at ON stash_items (stash_id, created_at);

CREATE INDEX idx_stash_items_data_version ON stash_items (data_version);

-- Item categories
CREATE TABLE
    stash_items_categories (
        stash_item_id BIGINT NOT NULL,
        --
        category TEXT NOT NULL,
        --
        deleted_at TIMESTAMPTZ,
        --
        FOREIGN KEY (stash_item_id) REFERENCES stash_items (stash_item_id) ON DELETE CASCADE
    );

CREATE INDEX idx_stash_items_categories_item_id ON stash_items_categories (stash_item_id);

CREATE INDEX idx_stash_items_categories_category ON stash_items_categories (category, stash_item_id)
WHERE
    deleted_at IS NULL;

-- Item stats
CREATE TABLE
    stash_items_stats (
        stash_item_id BIGINT NOT NULL,
        --
        item_stat JSONB NOT NULL,
        stat_modifier TEXT NOT NULL,
        stat_value DOUBLE PRECISION NOT NULL,
        --
        deleted_at TIMESTAMPTZ,
        --
        FOREIGN KEY (stash_item_id) REFERENCES stash_items (stash_item_id) ON DELETE CASCADE
    );

CREATE INDEX idx_stash_items_stats_item_id ON stash_items_stats (stash_item_id);

CREATE INDEX idx_stash_items_stats_item_stat ON stash_items_stats (
    item_stat,
    stat_modifier,
    stat_value,
    stash_item_id
)
WHERE
    deleted_at IS NULL;

-- New market table
CREATE TABLE
    market (
        market_id BIGSERIAL NOT NULL PRIMARY KEY,
        --
        stash_item_id BIGINT NOT NULL,
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
        item_damage_physical DOUBLE PRECISION,
        item_damage_fire DOUBLE PRECISION,
        item_damage_poison DOUBLE PRECISION,
        item_damage_storm DOUBLE PRECISION,
        item_crit_chance DOUBLE PRECISION,
        item_crit_damage DOUBLE PRECISION,
        --
        created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMPTZ,
        deleted_by UUID,
        --
        FOREIGN KEY (stash_item_id) REFERENCES stash_items (stash_item_id) ON DELETE CASCADE,
        FOREIGN KEY (recipient_id) REFERENCES users (user_id) ON DELETE SET NULL,
        FOREIGN KEY (deleted_by) REFERENCES users (user_id) ON DELETE SET NULL
    );

CREATE INDEX idx_market_item_id ON market (stash_item_id);

CREATE UNIQUE INDEX idx_market_item_id_active ON market (stash_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_recipient_id ON market (recipient_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_deleted ON market (deleted_at);

CREATE INDEX idx_market_base_item_id ON market (base_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_name ON market (UPPER(item_name))
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_price ON market (price)
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

CREATE INDEX idx_market_damage_physical ON market (item_damage_physical)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_fire ON market (item_damage_fire)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_poison ON market (item_damage_poison)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_storm ON market (item_damage_storm)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_crit_chance ON market (item_crit_chance)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_crit_damage ON market (item_crit_damage)
WHERE
    deleted_at IS NULL;