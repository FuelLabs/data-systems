CREATE TABLE IF NOT EXISTS outputs (
    subject TEXT PRIMARY KEY,
    block_height BIGINT,
    tx_id TEXT,
    tx_index INTEGER,
    output_index INTEGER,
    output_type TEXT,  -- 'coin', 'contract', 'change', 'variable', or 'contract_created'
    to_address TEXT,   -- for coin, change, and variable
    asset_id TEXT,     -- for coin, change, and variable
    contract_id TEXT,  -- for contract and contract_created
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_outputs_subject ON outputs (subject);
CREATE INDEX IF NOT EXISTS idx_outputs_block_height ON outputs (block_height);
CREATE INDEX IF NOT EXISTS idx_outputs_tx_id ON outputs (tx_id);
CREATE INDEX IF NOT EXISTS idx_outputs_tx_index ON outputs (tx_index);
CREATE INDEX IF NOT EXISTS idx_outputs_output_index ON outputs (output_index);
CREATE INDEX IF NOT EXISTS idx_outputs_output_type ON outputs (output_type);
CREATE INDEX IF NOT EXISTS idx_outputs_to_address ON outputs (to_address);
CREATE INDEX IF NOT EXISTS idx_outputs_asset_id ON outputs (asset_id);
CREATE INDEX IF NOT EXISTS idx_outputs_contract_id ON outputs (contract_id);
