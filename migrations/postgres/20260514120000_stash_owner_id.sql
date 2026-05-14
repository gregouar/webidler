ALTER TABLE
    stashes
ADD
    COLUMN owner_id UUID;

UPDATE
    stashes
SET
    owner_id = user_id
WHERE
    owner_id IS NULL;

ALTER TABLE
    stashes
ALTER COLUMN
    owner_id
SET
    NOT NULL;

CREATE INDEX idx_stashes_owner_id ON stashes (realm_id, owner_id);
