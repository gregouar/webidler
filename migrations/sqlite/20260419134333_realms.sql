-- Market filters
ALTER TABLE
    stash_items
ADD
    COLUMN item_cooldown REAL;

ALTER TABLE
    stash_items
ADD
    COLUMN item_upgrade_level INT;

UPDATE
    stash_items
SET
    item_upgrade_level = 0
WHERE
    item_upgrade_level IS NULL;

ALTER TABLE
    stash_items
ADD
    COLUMN item_power_level INT;

UPDATE
    stash_items
SET
    item_power_level = 0
WHERE
    item_power_level IS NULL;

ALTER TABLE
    market
ADD
    COLUMN item_cooldown REAL;

ALTER TABLE
    market
ADD
    COLUMN item_upgrade_level INT;

UPDATE
    market
SET
    item_upgrade_level = 0
WHERE
    item_upgrade_level IS NULL;

ALTER TABLE
    market
ADD
    COLUMN item_power_level INT;

UPDATE
    market
SET
    item_power_level = 0
WHERE
    item_power_level IS NULL;

-- SSF
ALTER TABLE
    characters
ADD
    COLUMN is_ssf BOOLEAN;

UPDATE
    characters
SET
    is_ssf = false
WHERE
    is_ssf IS NULL;

-- Realms
ALTER TABLE
    characters
ADD
    COLUMN realm_id TEXT;

UPDATE
    characters
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

ALTER TABLE
    game_stats
ADD
    COLUMN realm_id TEXT;

UPDATE
    game_stats
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

ALTER TABLE
    stashes
ADD
    COLUMN realm_id TEXT;

UPDATE
    stashes
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

ALTER TABLE
    stash_items_categories
ADD
    COLUMN realm_id TEXT;

UPDATE
    stash_items_categories
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

ALTER TABLE
    stash_items_stats
ADD
    COLUMN realm_id TEXT;

UPDATE
    stash_items_stats
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

ALTER TABLE
    market
ADD
    COLUMN realm_id TEXT;

UPDATE
    market
SET
    realm_id = 'Legacy'
WHERE
    realm_id IS NULL;

DROP INDEX IF EXISTS idx_stashes_user_id;

DROP INDEX IF EXISTS idx_stashes_type;

DROP INDEX IF EXISTS idx_stash_items_categories_category;

DROP INDEX IF EXISTS idx_stash_items_stats_item_stat;

DROP INDEX IF EXISTS idx_market_character_id;

DROP INDEX IF EXISTS idx_market_main_filters;

DROP INDEX IF EXISTS idx_market_extra_filters;

DROP INDEX IF EXISTS idx_market_item_id;

DROP INDEX IF EXISTS idx_market_item_id_active;

DROP INDEX IF EXISTS idx_market_recipient_id;

DROP INDEX IF EXISTS idx_market_deleted;

DROP INDEX IF EXISTS idx_market_base_item_id;

DROP INDEX IF EXISTS idx_market_name;

DROP INDEX IF EXISTS idx_market_price;

DROP INDEX IF EXISTS idx_market_rarity;

DROP INDEX IF EXISTS idx_market_level;

DROP INDEX IF EXISTS idx_market_armor;

DROP INDEX IF EXISTS idx_market_block;

DROP INDEX IF EXISTS idx_market_damages;

DROP INDEX IF EXISTS idx_market_damage_physical;

DROP INDEX IF EXISTS idx_market_damage_fire;

DROP INDEX IF EXISTS idx_market_damage_poison;

DROP INDEX IF EXISTS idx_market_damage_storm;

DROP INDEX IF EXISTS idx_market_crit_chance;

DROP INDEX IF EXISTS idx_market_crit_damage;

DROP INDEX IF EXISTS idx_market_data_version;

DROP INDEX IF EXISTS idx_market_stats_market_id;

DROP INDEX IF EXISTS idx_market_stats_item_stat;

CREATE INDEX idx_stashes_user_id ON stashes (realm_id, user_id);

CREATE INDEX idx_stash_items_cooldown ON stash_items (stash_id, item_cooldown)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_stash_items_upgrade_level ON stash_items (stash_id, item_upgrade_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_stash_items_power_level ON stash_items (stash_id, item_power_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_stash_items_categories_category ON stash_items_categories (realm_id, category, stash_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_stash_items_stats_item_stat ON stash_items_stats (
    realm_id,
    item_stat,
    stat_modifier,
    stat_value,
    stash_item_id
)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_item_id ON market (realm_id, stash_item_id);

CREATE UNIQUE INDEX idx_market_item_id_active ON market (realm_id, stash_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_recipient_id ON market (realm_id, recipient_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_deleted ON market (realm_id, deleted_at);

CREATE INDEX idx_market_base_item_id ON market (realm_id, base_item_id)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_name ON market (realm_id, UPPER(item_name))
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_price ON market (realm_id, price)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_rarity ON market (realm_id, item_rarity)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_level ON market (realm_id, item_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_armor ON market (realm_id, item_armor)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_block ON market (realm_id, item_block)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damages ON market (realm_id, item_damages)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_physical ON market (realm_id, item_damage_physical)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_fire ON market (realm_id, item_damage_fire)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_poison ON market (realm_id, item_damage_poison)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_damage_storm ON market (realm_id, item_damage_storm)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_crit_chance ON market (realm_id, item_crit_chance)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_crit_damage ON market (realm_id, item_crit_damage)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_cooldown ON market (realm_id, item_cooldown)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_upgrade_level ON market (realm_id, item_upgrade_level)
WHERE
    deleted_at IS NULL;

CREATE INDEX idx_market_power_level ON market (realm_id, item_power_level)
WHERE
    deleted_at IS NULL;

-- Leaderboard
CREATE TABLE leaderboard (
    character_id TEXT NOT NULL,
    realm_id TEXT NOT NULL,
    area_id TEXT NOT NULL,
    -- 
    area_level INT NOT NULL,
    elapsed_time REAL NOT NULL,
    --
    data_version TEXT,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --
    PRIMARY KEY (character_id, realm_id, area_id)
);

CREATE INDEX idx_leaderboard_realm_area_rank ON leaderboard (
    realm_id,
    area_id,
    area_level DESC,
    elapsed_time ASC,
    updated_at ASC
);

INSERT INTO
    leaderboard (
        character_id,
        realm_id,
        area_id,
        area_level,
        elapsed_time,
        data_version,
        created_at,
        updated_at
    )
SELECT
    best_runs.character_id,
    'Legacy' AS realm_id,
    best_runs.area_id,
    best_runs.area_level,
    best_runs.elapsed_time,
    best_runs.data_version,
    best_runs.created_at,
    best_runs.created_at
FROM
    (
        SELECT
            gs.character_id,
            gs.area_id,
            gs.area_level,
            gs.elapsed_time,
            gs.data_version,
            gs.created_at,
            ROW_NUMBER() OVER (
                PARTITION BY gs.area_id,
                gs.character_id
                ORDER BY
                    gs.area_level DESC,
                    gs.elapsed_time ASC,
                    gs.created_at ASC
            ) AS best_rank
        FROM
            game_stats gs
        WHERE
            gs.data_version >= '0.1.9'
            OR gs.data_version = '0.1.8'
            OR gs.data_version = '0.1.8_0.1.7'
    ) AS best_runs
WHERE
    best_runs.best_rank = 1
    AND (
        best_runs.elapsed_time IS NOT NULL
        AND best_runs.elapsed_time <> 0
    )
    AND best_runs.area_level <> 0;