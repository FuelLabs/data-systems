CREATE TABLE IF NOT EXISTS utxos (
    _id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    input_index INTEGER NOT NULL,
    utxo_type TEXT NOT NULL,    -- 'message', 'coin', or 'contract'
    utxo_id TEXT NOT NULL,      -- hex string of the UTXO identifier
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_utxos_subject ON utxos (subject);
CREATE INDEX IF NOT EXISTS idx_utxos_block_height ON utxos (block_height);
CREATE INDEX IF NOT EXISTS idx_utxos_tx_id ON utxos (tx_id);
CREATE INDEX IF NOT EXISTS idx_utxos_tx_index ON utxos (tx_index);
CREATE INDEX IF NOT EXISTS idx_utxos_input_index ON utxos (input_index);
CREATE INDEX IF NOT EXISTS idx_utxos_utxo_type ON utxos (utxo_type);
CREATE INDEX IF NOT EXISTS idx_utxos_utxo_id ON utxos (utxo_id);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_utxos_utxo_type_block_height ON utxos (utxo_type, block_height);
CREATE INDEX IF NOT EXISTS idx_utxos_utxo_id_block_height ON utxos (utxo_id, block_height);

-- Composite index for ordering by (block_height, tx_index, input_index)
CREATE INDEX IF NOT EXISTS idx_utxos_ordering ON utxos (block_height, tx_index, input_index);
