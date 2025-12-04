-- Add elemental damage columns to market
ALTER TABLE market
ADD COLUMN item_damage_physical REAL;

ALTER TABLE market
ADD COLUMN item_damage_fire REAL;

ALTER TABLE market
ADD COLUMN item_damage_poison REAL;

ALTER TABLE market
ADD COLUMN item_damage_storm REAL;

-- Add new indexes on market
DROP INDEX idx_market_name;

CREATE INDEX idx_market_name ON market (UPPER(item_name))
WHERE
    deleted_at IS NULL;

DROP INDEX idx_market_main_filters;

CREATE INDEX idx_market_price ON market (price)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_rarity ON market (item_rarity)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_level ON market (item_level)
WHERE
    deleted_at IS NULL;

DROP INDEX idx_market_extra_filters;

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

CREATE INDEX idx_market_updated_at ON market (updated_at);

DROP INDEX idx_market_categories_category;

CREATE INDEX idx_market_categories_category ON market_categories (category, market_id)
WHERE
    deleted_at IS NULL;

DROP INDEX idx_market_stats_item_stat;

CREATE INDEX idx_market_stats_item_stat ON market_stats (item_stat, stat_modifier, stat_value, market_id)
WHERE
    deleted_at IS NULL;

-- User Stashes Table
CREATE TABLE
    user_stashes (
        item_id INTEGER NOT NULL PRIMARY KEY,
        --
        user_id TEXT NOT NULL,
        --
        base_item_id TEXT NOT NULL,
        item_name TEXT NOT NULL,
        item_rarity TEXT NOT NULL,
        item_level INT NOT NULL,
        item_armor REAL,
        item_block REAL,
        item_damages REAL,
        item_damage_physical REAL,
        item_damage_fire REAL,
        item_damage_poison REAL,
        item_damage_storm REAL,
        item_crit_chance REAL,
        item_crit_damage REAL,
        -- 
        item_data TEXT NOT NULL,
        data_version TEXT NOT NULL,
        --
        created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        deleted_at TIMESTAMP,
        --
        FOREIGN KEY (user_id) REFERENCES users (user_id) ON DELETE CASCADE
    );

CREATE INDEX idx_user_stashes_user_id ON user_stashes (user_id, deleted_at);

CREATE INDEX idx_user_stashes_base_item_id ON user_stashes (user_id, base_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_name ON user_stashes (user_id, UPPER(item_name))
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_rarity ON user_stashes (user_id, item_rarity)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_level ON user_stashes (user_id, item_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_armor ON user_stashes (user_id, item_armor)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_block ON user_stashes (user_id, item_block)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_damages ON user_stashes (user_id, item_damages)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_damage_physical ON user_stashes (user_id, item_damage_physical)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_damage_fire ON user_stashes (user_id, item_damage_fire)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_damage_poison ON user_stashes (user_id, item_damage_poison)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_damage_storm ON user_stashes (user_id, item_damage_storm)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_crit_chance ON user_stashes (user_id, item_crit_chance)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_crit_damage ON user_stashes (user_id, item_crit_damage)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_user_stashes_updated_at ON user_stashes (user_id, updated_at);

CREATE INDEX idx_user_stashes_data_version ON user_stashes (data_version);

-- Item categories
CREATE TABLE
    user_stashes_categories (
        item_id INTEGER NOT NULL,
        category TEXT NOT NULL,
        deleted_at TIMESTAMP,
        FOREIGN KEY (item_id) REFERENCES user_stashes (item_id) ON DELETE CASCADE
    );

CREATE INDEX idx_user_stashes_categories_item_id ON user_stashes_categories (item_id);

CREATE INDEX idx_user_stashes_categories_category ON user_stashes_categories (category, item_id)
WHERE
    deleted_at IS NULL;

-- Item stats
CREATE TABLE
    user_stashes_stats (
        item_id INTEGER NOT NULL,
        item_stat TEXT NOT NULL,
        stat_modifier TEXT NOT NULL,
        stat_value REAL NOT NULL,
        deleted_at TIMESTAMP,
        FOREIGN KEY (item_id) REFERENCES user_stashes (item_id) ON DELETE CASCADE
    );

CREATE INDEX idx_user_stashes_stats_item_id ON user_stashes_stats (item_id);

CREATE INDEX idx_user_stashes_stats_item_stat ON user_stashes_stats (item_stat, stat_modifier, stat_value, item_id)
WHERE
    deleted_at IS NULL;