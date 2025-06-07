CREATE TABLE leaderboard (
    id BIGSERIAL PRIMARY KEY,
    player_name VARCHAR(200) NOT NULL,
    area_level BIGINT NOT NULL,
    time_played_seconds BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    comments TEXT NOT NULL
);

CREATE INDEX idx_leaderboard_highest_level ON leaderboard (
    area_level DESC,
    time_played_seconds ASC,
    created_at ASC
);