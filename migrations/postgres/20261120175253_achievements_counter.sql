CREATE TABLE character_achievements_counters (
    character_id UUID NOT NULL,
    counter_id TEXT NOT NULL,
    amount BIGINT NOT NULL DEFAULT 0 CHECK (amount >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (character_id, counter_id),
    FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE CASCADE
);

ALTER TABLE character_area_completed ADD COLUMN player_deaths BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN elapsed_time_seconds BIGINT NOT NULL DEFAULT 0;
