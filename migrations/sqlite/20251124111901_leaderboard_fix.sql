DROP INDEX idx_game_stats_leaderboard;
CREATE INDEX idx_game_stats_leaderboard ON game_stats (area_id,character_id,area_level DESC,elapsed_time ASC,created_at ASC);