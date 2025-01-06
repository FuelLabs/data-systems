-- Create records table
CREATE TABLE IF NOT EXISTS records (
    subject TEXT PRIMARY KEY,
    value BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create index for subject lookups
CREATE INDEX IF NOT EXISTS idx_records_subject ON records (subject);
