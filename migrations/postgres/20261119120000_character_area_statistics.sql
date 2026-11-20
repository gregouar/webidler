ALTER TABLE character_area_completed ADD COLUMN monsters_killed BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN champion_monsters_killed BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN bosses_killed BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN areas_completed BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN edicts_spent BIGINT NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN gems_obtained BIGINT NOT NULL DEFAULT 0;
