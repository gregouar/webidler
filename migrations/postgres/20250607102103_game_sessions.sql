CREATE TABLE game_sessions (
    session_id SERIAL,
    user_id VARCHAR(200) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ DEFAULT NULL,
    UNIQUE(user_id, ended_at)
);

CREATE INDEX idx_session_user_id ON game_sessions (user_id, ended_at);

DROP TABLE leaderboard;

CREATE TABLE leaderboard (
    session_id INT NOT NULL PRIMARY KEY,
    player_name VARCHAR(200) NOT NULL,
    area_level INT NOT NULL,
    time_played_seconds INT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    comments TEXT NOT NULL
);

CREATE INDEX idx_leaderboard_highest_level ON leaderboard (
    area_level DESC,
    time_played_seconds ASC,
    created_at ASC
);