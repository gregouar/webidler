ALTER TABLE character_area_completed
ADD COLUMN max_power_shard_level INTEGER NOT NULL DEFAULT 0;

ALTER TABLE stash_items
ADD COLUMN max_power_shard_level INTEGER;

ALTER TABLE market
ADD COLUMN max_power_shard_level INTEGER;

CREATE INDEX idx_stash_items_max_power_shard_level ON stash_items (stash_id, max_power_shard_level);

CREATE INDEX idx_market_max_power_shard_level ON market (realm_id, max_power_shard_level);