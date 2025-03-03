-- Create records table
CREATE TABLE IF NOT EXISTS blocks (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    block_da_height BIGINT NOT NULL,
    block_height BIGINT NOT NULL,
    producer_address TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    block_propagation_ms INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_subject ON blocks (subject);
CREATE INDEX IF NOT EXISTS idx_blocks_producer_address ON blocks (producer_address);
CREATE INDEX IF NOT EXISTS idx_blocks_block_da_height ON blocks (block_da_height);
CREATE INDEX IF NOT EXISTS idx_blocks_block_height ON blocks (block_height);
CREATE INDEX IF NOT EXISTS idx_blocks_created_at ON blocks (created_at);
CREATE INDEX IF NOT EXISTS idx_blocks_published_at ON blocks (published_at);
