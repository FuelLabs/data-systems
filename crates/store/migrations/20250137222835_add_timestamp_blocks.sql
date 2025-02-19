ALTER TABLE blocks
ADD COLUMN timestamp TIMESTAMP WITH TIME ZONE;

CREATE INDEX IF NOT EXISTS idx_blocks_timestamp ON blocks (timestamp);
