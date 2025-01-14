-- Create records table
CREATE TABLE IF NOT EXISTS blocks (
    _id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    producer_address TEXT NOT NULL,
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_subject ON blocks (subject);
CREATE INDEX IF NOT EXISTS idx_blocks_producer_address ON blocks (producer_address);
CREATE INDEX IF NOT EXISTS idx_blocks_block_height ON blocks (block_height);
