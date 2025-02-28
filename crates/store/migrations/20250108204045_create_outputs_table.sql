CREATE TABLE IF NOT EXISTS outputs (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    output_index INTEGER NOT NULL,
    output_type TEXT NOT NULL,  -- 'coin', 'contract', 'change', 'variable', or 'contract_created'
    to_address TEXT,   -- for coin, change, and variable
    asset_id TEXT,     -- for coin, change, and variable
    contract_id TEXT,  -- for contract and contract_created
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
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
CREATE INDEX IF NOT EXISTS idx_outputs_created_at ON outputs (created_at);
CREATE INDEX IF NOT EXISTS idx_outputs_published_at ON outputs (published_at);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_outputs_output_type_block_height ON outputs (output_type, block_height);
CREATE INDEX IF NOT EXISTS idx_outputs_to_address_block_height ON outputs (to_address, block_height);
CREATE INDEX IF NOT EXISTS idx_outputs_asset_id_block_height ON outputs (asset_id, block_height);
CREATE INDEX IF NOT EXISTS idx_outputs_contract_id_block_height ON outputs (contract_id, block_height);

-- Composite index for ordering by (block_height, tx_index, output_index)
CREATE INDEX IF NOT EXISTS idx_outputs_ordering ON outputs (block_height, tx_index, output_index);
