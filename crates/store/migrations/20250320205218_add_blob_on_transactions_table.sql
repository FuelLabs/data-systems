-- Add migration script here
CREATE TYPE "TransactionType" AS ENUM (
    'SCRIPT',
    'CREATE',
    'MINT',
    'UPGRADE',
    'UPLOAD',
    'BLOB'
);

ALTER TABLE transactions
ADD COLUMN tx_type "TransactionType",
ADD COLUMN blob_id TEXT UNIQUE;

CREATE INDEX IF NOT EXISTS idx_transactions_tx_type ON transactions(tx_type);
CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions(blob_id);
