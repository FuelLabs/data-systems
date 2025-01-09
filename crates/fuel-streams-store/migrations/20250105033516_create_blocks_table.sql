-- Create records table
CREATE TABLE IF NOT EXISTS blocks (
    subject TEXT PRIMARY KEY,
    producer_address TEXT,
    height BIGINT NOT NULL,
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_subject ON blocks (subject);
CREATE INDEX IF NOT EXISTS idx_blocks_producer_address ON blocks (producer_address);
CREATE INDEX IF NOT EXISTS idx_blocks_height ON blocks (height);