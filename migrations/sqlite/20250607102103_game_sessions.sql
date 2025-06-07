CREATE TABLE game_sessions (
    session_id INTEGER PRIMARY KEY AUTOINCREMENT,
    user_id STRING,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP NULL DEFAULT NULL,

    UNIQUE(user_id, ended_at)
);

CREATE INDEX idx_session_user_id ON game_sessions (user_id, ended_at);

DROP TABLE leaderboard;

CREATE TABLE leaderboard (
    session_id INTEGER PRIMARY KEY,
    player_name TEXT NOT NULL,
    area_level INTEGER NOT NULL,
    time_played_seconds INTEGER NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    comments TEXT NOT NULL
);

CREATE INDEX idx_leaderboard_highest_level ON leaderboard (area_level, time_played_seconds, created_at);