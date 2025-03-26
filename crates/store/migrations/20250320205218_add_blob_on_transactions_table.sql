ALTER TABLE transactions
ADD COLUMN blob_id TEXT;

CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions(blob_id);
