CREATE TABLE IF NOT EXISTS inputs (
    id SERIAL PRIMARY KEY,
    cursor TEXT NOT NULL, -- {block_height}-{tx_index}-{input_index}
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    input_index INTEGER NOT NULL,
    input_type TEXT NOT NULL,  -- 'coin', 'contract', or 'message'
    owner_id TEXT,    -- for coin
    asset_id TEXT,    -- for coin
    contract_id TEXT, -- for contract
    sender_address TEXT,      -- for message
    recipient_address TEXT,   -- for message
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_inputs_subject ON inputs (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_cursor ON transactions (cursor);
CREATE INDEX IF NOT EXISTS idx_inputs_block_height ON inputs (block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_tx_id ON inputs (tx_id);
CREATE INDEX IF NOT EXISTS idx_inputs_tx_index ON inputs (tx_index);
CREATE INDEX IF NOT EXISTS idx_inputs_input_index ON inputs (input_index);
CREATE INDEX IF NOT EXISTS idx_inputs_input_type ON inputs (input_type);
CREATE INDEX IF NOT EXISTS idx_inputs_owner_id ON inputs (owner_id);
CREATE INDEX IF NOT EXISTS idx_inputs_asset_id ON inputs (asset_id);
CREATE INDEX IF NOT EXISTS idx_inputs_contract_id ON inputs (contract_id);
CREATE INDEX IF NOT EXISTS idx_inputs_sender_address ON inputs (sender_address);
CREATE INDEX IF NOT EXISTS idx_inputs_recipient_address ON inputs (recipient_address);
CREATE INDEX IF NOT EXISTS idx_inputs_created_at ON inputs (created_at);
CREATE INDEX IF NOT EXISTS idx_inputs_published_at ON inputs (published_at);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_inputs_input_type_block_height ON inputs (input_type, block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_owner_id_block_height ON inputs (owner_id, block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_asset_id_block_height ON inputs (asset_id, block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_contract_id_block_height ON inputs (contract_id, block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_sender_address_block_height ON inputs (sender_address, block_height);
CREATE INDEX IF NOT EXISTS idx_inputs_recipient_address_block_height ON inputs (recipient_address, block_height);

-- Composite index for ordering by (block_height, tx_index, input_index)
CREATE INDEX IF NOT EXISTS idx_inputs_ordering ON inputs (block_height, tx_index, input_index);
