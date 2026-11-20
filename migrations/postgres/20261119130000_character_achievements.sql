CREATE TABLE character_achievements (
    character_id UUID NOT NULL,
    achievement_id TEXT NOT NULL,
    data_version TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (character_id, achievement_id),
    FOREIGN KEY (character_id) REFERENCES characters (character_id) ON DELETE CASCADE
);

CREATE INDEX idx_character_achievements_data_version
ON character_achievements (data_version);
