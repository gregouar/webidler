
CREATE TABLE game_stats (
    session_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- 
    character_id TEXT NOT NULL,
    -- 
    area_id TEXT NOT NULL,
    area_level INT NOT NULL,
    elapsed_time REAL,
    --
    stats_data TEXT,
    items_data TEXT,
    passives_data TEXT,
    skills_data TEXT,
    data_version TEXT,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_game_stats_character_id ON game_stats (character_id,created_at);
CREATE INDEX idx_game_stats_leaderboard ON game_stats (area_id,area_level,created_at,elapsed_time);
CREATE INDEX idx_game_stats_data_version ON game_stats (data_version);
