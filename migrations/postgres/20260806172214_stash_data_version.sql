ALTER TABLE stashes
ADD COLUMN data_version TEXT NOT NULL DEFAULT '0.0.00';

CREATE INDEX idx_stashes_data_version ON stashes (data_version);
