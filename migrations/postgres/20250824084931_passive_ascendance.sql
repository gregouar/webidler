ALTER TABLE
    characters_data
ADD
    COLUMN passives_data BYTEA;

-- Reset for easy migration yek yek
TRUNCATE TABLE character_area_completed;

TRUNCATE TABLE saved_game_instances;

UPDATE
    TABLE characters
SET
    resource_gems = 0,
    resource_shards = 0;