
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
CREATE INDEX idx_game_stats_leaderboard ON game_stats (area_id,character_id,area_level DESC,elapsed_time DESC,created_at);
CREATE INDEX idx_game_stats_data_version ON game_stats (data_version);

INSERT INTO game_stats 
    (
        character_id,
        area_id,
        area_level,
        elapsed_time,
        stats_data,
        items_data,
        passives_data,
        skills_data,
        data_version,
        created_at
    )
    SELECT 
        character_id,
        area_id,
        max_area_level,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        NULL,
        updated_at
    FROM 
        character_area_completed;
