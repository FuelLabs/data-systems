CREATE TYPE IF NOT EXISTS record_entity AS ENUM (
    'block',
    'transaction',
    'input',
    'output',
    'receipt',
    'log',
    'utxo'
);

-- Create records table
CREATE TABLE IF NOT EXISTS records (
    subject TEXT PRIMARY KEY,
    entity record_entity NOT NULL,
    sequence_order INT4 NOT NULL,
    value BYTES NOT NULL
);

-- Create index for subject lookups
CREATE INDEX IF NOT EXISTS idx_records_subject ON records (subject);
