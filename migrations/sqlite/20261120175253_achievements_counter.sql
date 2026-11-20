CREATE TABLE character_achievements_counters (
    character_id TEXT NOT NULL,
    counter_id TEXT NOT NULL,
    amount INTEGER NOT NULL DEFAULT 0 CHECK (amount >= 0),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (character_id, counter_id),
    FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE CASCADE
);

ALTER TABLE character_area_completed ADD COLUMN player_deaths INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN elapsed_time_seconds INTEGER NOT NULL DEFAULT 0;
