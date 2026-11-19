ALTER TABLE character_area_completed ADD COLUMN monsters_killed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN champion_monsters_killed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN bosses_killed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN areas_completed INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN edicts_spent INTEGER NOT NULL DEFAULT 0;
ALTER TABLE character_area_completed ADD COLUMN gems_obtained INTEGER NOT NULL DEFAULT 0;
