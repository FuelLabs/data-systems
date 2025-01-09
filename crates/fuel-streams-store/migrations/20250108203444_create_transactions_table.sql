CREATE TABLE IF NOT EXISTS transactions (
    subject TEXT PRIMARY KEY,
    block_height BIGINT,
    tx_id TEXT,
    tx_index INTEGER,
    tx_status TEXT,
    kind TEXT,
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_subject ON transactions (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_id ON transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_index ON transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status ON transactions (tx_status);
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions (kind);
