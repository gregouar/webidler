ALTER TABLE
    market
ADD
    COLUMN data_version TEXT;

CREATE INDEX idx_market_data_version ON market (data_version);