CREATE TABLE IF NOT EXISTS receipts (
    subject TEXT PRIMARY KEY,
    block_height BIGINT,
    tx_id TEXT,
    tx_index INTEGER,
    receipt_index INTEGER,
    receipt_type TEXT,         -- 'call', 'return', 'return_data', 'panic', 'revert', 'log', 'log_data',
                               -- 'transfer', 'transfer_out', 'script_result', 'message_out', 'mint', 'burn'
    from_contract_id TEXT,     -- ContractId for call/transfer/transfer_out
    to_contract_id TEXT,       -- ContractId for call/transfer
    to_address TEXT,           -- Address for transfer_out
    asset_id TEXT,             -- for call/transfer/transfer_out
    contract_id TEXT,          -- ContractId for return/return_data/panic/revert/log/log_data/mint/burn
    sub_id TEXT,               -- for mint/burn
    sender_address TEXT,       -- Address for message_out
    recipient_address TEXT,    -- Address for message_out
    value BYTEA NOT NULL
);

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
