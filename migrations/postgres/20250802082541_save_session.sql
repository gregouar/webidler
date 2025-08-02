CREATE TABLE saved_game_instances (
    user_id VARCHAR(200) NOT NULL PRIMARY KEY,
    saved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    game_data bytea NOT NULL
);