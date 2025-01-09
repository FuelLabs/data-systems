CREATE TABLE IF NOT EXISTS transactions (
    _id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    tx_status TEXT NOT NULL,
    kind TEXT NOT NULL,
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_transactions_subject ON transactions (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_id ON transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_index ON transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status ON transactions (tx_status);
CREATE INDEX IF NOT EXISTS idx_transactions_kind ON transactions (kind);
