CREATE TABLE IF NOT EXISTS predicates (
    id SERIAL PRIMARY KEY,
    blob_id TEXT,
    predicate_address TEXT UNIQUE NOT NULL,
    predicate_bytecode BYTEA NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_predicates_blob_id ON predicates (blob_id);
CREATE INDEX IF NOT EXISTS idx_predicates_predicate_address ON predicates (predicate_address);
CREATE INDEX IF NOT EXISTS idx_predicates_created_at ON predicates (created_at);
CREATE INDEX IF NOT EXISTS idx_predicates_published_at ON predicates (published_at);

CREATE TABLE IF NOT EXISTS predicate_transactions (
    subject TEXT UNIQUE NOT NULL,
    predicate_id INTEGER REFERENCES predicates(id),
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    input_index INTEGER NOT NULL,
    PRIMARY KEY (predicate_id, tx_id)
);

CREATE INDEX IF NOT EXISTS idx_predicate_transactions_subject ON predicate_transactions (subject);
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_tx_index ON predicate_transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_input_index ON predicate_transactions (input_index);
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_block_height ON predicate_transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_tx_id ON predicate_transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_predicate_id ON predicate_transactions (predicate_id);

-- Composite index for ordering by (block_height, tx_index, input_index)
CREATE INDEX IF NOT EXISTS idx_predicate_transactions_ordering ON predicate_transactions (block_height, tx_index, input_index);
