CREATE TABLE IF NOT EXISTS inputs (
    _id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    input_index INTEGER NOT NULL,
    input_type TEXT NOT NULL,  -- 'coin', 'contract', or 'message'
    owner_id TEXT,    -- for coin
    asset_id TEXT,    -- for coin
    contract_id TEXT, -- for contract
    sender TEXT,      -- for message
    recipient TEXT,   -- for message
    value BYTEA NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_inputs_subject ON inputs (subject);
CREATE INDEX IF NOT EXISTS idx_inputs_block_height ON inputs (block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_tx_id ON inputs (tx_id);
CREATE INDEX IF NOT EXISTS idx_inputs_tx_index ON inputs (tx_index);
CREATE INDEX IF NOT EXISTS idx_inputs_input_index ON inputs (input_index);
CREATE INDEX IF NOT EXISTS idx_inputs_input_type ON inputs (input_type);
CREATE INDEX IF NOT EXISTS idx_inputs_owner_id ON inputs (owner_id);
CREATE INDEX IF NOT EXISTS idx_inputs_asset_id ON inputs (asset_id);
CREATE INDEX IF NOT EXISTS idx_inputs_contract_id ON inputs (contract_id);
CREATE INDEX IF NOT EXISTS idx_inputs_sender ON inputs (sender);
CREATE INDEX IF NOT EXISTS idx_inputs_recipient ON inputs (recipient);
