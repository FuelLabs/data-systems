CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    cursor TEXT NOT NULL, -- {block_height}-{tx_index}
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL UNIQUE,
    tx_index INTEGER NOT NULL,
    tx_status TEXT NOT NULL,
    type TEXT NOT NULL,
    blob_id TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_subject ON transactions (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_cursor ON transactions (cursor);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_id ON transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_index ON transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status ON transactions (tx_status);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (type);
CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions (blob_id);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_published_at ON transactions (published_at);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status_block_height ON transactions (tx_status, block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_type_block_height ON transactions (type, block_height);

-- Composite index for ordering by (block_height, tx_index)
CREATE INDEX IF NOT EXISTS idx_transactions_ordering ON transactions (block_height, tx_index);
