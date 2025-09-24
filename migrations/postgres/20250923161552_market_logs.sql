ALTER TABLE market
ADD COLUMN item_crit_chance DOUBLE PRECISION;

ALTER TABLE market
ADD COLUMN item_crit_damage DOUBLE PRECISION;

ALTER TABLE market
ADD COLUMN deleted_by UUID;

ALTER TABLE market ADD CONSTRAINT fk_deleted_by FOREIGN KEY (deleted_by) REFERENCES characters (character_id) ON DELETE SET NULL;