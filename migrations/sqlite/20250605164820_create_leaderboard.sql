CREATE TABLE leaderboard (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    player_name TEXT NOT NULL,
    area_level INTEGER NOT NULL,
    time_played_seconds INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    comments TEXT NOT NULL
);

CREATE INDEX idx_leaderboard_highest_level ON leaderboard (area_level, time_played_seconds, created_at);