CREATE TABLE IF NOT EXISTS receipts (
    id SERIAL PRIMARY KEY,
    cursor TEXT NOT NULL, -- {block_height}-{tx_index}-{receipt_index}
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    block_height BIGINT NOT NULL,
    tx_id TEXT NOT NULL,
    tx_index INTEGER NOT NULL,
    receipt_index INTEGER NOT NULL,
    receipt_type TEXT NOT NULL,         -- 'call', 'return', 'return_data', 'panic', 'revert', 'log', 'log_data',
                                       -- 'transfer', 'transfer_out', 'script_result', 'message_out', 'mint', 'burn'
    from_contract_id TEXT,     -- ContractId for call/transfer/transfer_out
    to_contract_id TEXT,       -- ContractId for call/transfer
    to_address TEXT,           -- Address for transfer_out
    asset_id TEXT,             -- for call/transfer/transfer_out
    contract_id TEXT,          -- ContractId for return/return_data/panic/revert/log/log_data/mint/burn
    sub_id TEXT,               -- for mint/burn
    sender_address TEXT,       -- Address for message_out
    recipient_address TEXT,    -- Address for message_out
    created_at TIMESTAMP WITH TIME ZONE NOT NULL,
    published_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_receipts_cursor ON receipts (cursor);
CREATE INDEX IF NOT EXISTS idx_receipts_subject ON receipts (subject);
CREATE INDEX IF NOT EXISTS idx_receipts_block_height ON receipts (block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_tx_id ON receipts (tx_id);
CREATE INDEX IF NOT EXISTS idx_receipts_tx_index ON receipts (tx_index);
CREATE INDEX IF NOT EXISTS idx_receipts_receipt_index ON receipts (receipt_index);
CREATE INDEX IF NOT EXISTS idx_receipts_receipt_type ON receipts (receipt_type);
CREATE INDEX IF NOT EXISTS idx_receipts_from_contract_id ON receipts (from_contract_id);
CREATE INDEX IF NOT EXISTS idx_receipts_to_contract_id ON receipts (to_contract_id);
CREATE INDEX IF NOT EXISTS idx_receipts_to_address ON receipts (to_address);
CREATE INDEX IF NOT EXISTS idx_receipts_asset_id ON receipts (asset_id);
CREATE INDEX IF NOT EXISTS idx_receipts_contract_id ON receipts (contract_id);
CREATE INDEX IF NOT EXISTS idx_receipts_sub_id ON receipts (sub_id);
CREATE INDEX IF NOT EXISTS idx_receipts_sender_address ON receipts (sender_address);
CREATE INDEX IF NOT EXISTS idx_receipts_recipient_address ON receipts (recipient_address);
CREATE INDEX IF NOT EXISTS idx_receipts_created_at ON receipts (created_at);
CREATE INDEX IF NOT EXISTS idx_receipts_published_at ON receipts (published_at);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_receipts_receipt_type_block_height ON receipts (receipt_type, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_from_contract_id_block_height ON receipts (from_contract_id, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_to_contract_id_block_height ON receipts (to_contract_id, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_to_address_block_height ON receipts (to_address, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_asset_id_block_height ON receipts (asset_id, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_contract_id_block_height ON receipts (contract_id, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_sub_id_block_height ON receipts (sub_id, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_sender_address_block_height ON receipts (sender_address, block_height);
CREATE INDEX IF NOT EXISTS idx_receipts_recipient_address_block_height ON receipts (recipient_address, block_height);

-- Composite index for ordering by (block_height, tx_index, receipt_index)
CREATE INDEX IF NOT EXISTS idx_receipts_ordering ON receipts (block_height, tx_index, receipt_index);
