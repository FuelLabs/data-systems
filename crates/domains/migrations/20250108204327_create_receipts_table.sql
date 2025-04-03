CREATE TYPE "receipt_type" AS ENUM (
    'call',
    'return',
    'return_data',
    'panic',
    'revert',
    'log',
    'log_data',
    'transfer',
    'transfer_out',
    'script_result',
    'message_out',
    'mint',
    'burn'
);

-- ------------------------------------------------------------------------------
-- Receipts table
-- ------------------------------------------------------------------------------
CREATE TABLE "receipts" (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT UNIQUE NOT NULL,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_index" INTEGER NOT NULL,
    "receipt_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}-{receipt_index}
    -- common props
    "type" receipt_type NOT NULL,

    -- call/transfer shared props
    "from_contract_id" TEXT,      -- 'id' in types
    "to_contract_id" TEXT,        -- 'to' in types
    "amount" BIGINT,
    "asset_id" TEXT,
    "gas" BIGINT,                 -- call specific
    "param1" BIGINT,              -- call specific
    "param2" BIGINT,              -- call specific

    -- return/return_data/panic/revert/log/log_data shared props
    "contract_id" TEXT,           -- 'id' in types
    "pc" BIGINT,
    "is" BIGINT,

    -- return specific props
    "val" BIGINT,

    -- return_data/log_data shared props
    "ptr" BIGINT,
    "len" BIGINT,
    "digest" TEXT,
    "data" TEXT,

    -- log specific props
    "ra" BIGINT,
    "rb" BIGINT,
    "rc" BIGINT,
    "rd" BIGINT,

    -- transfer_out specific props
    "to_address" TEXT,            -- 'to' in types for transfer_out

    -- script_result specific props
    "panic_reason" TEXT,               -- panic specific: reason
    "panic_instruction" INTEGER,         -- panic specific: instruction
    "result" TEXT,                -- script_result specific
    "gas_used" BIGINT,

    -- message_out specific props
    "sender_address" TEXT,        -- 'sender' in types
    "recipient_address" TEXT,     -- 'recipient' in types
    "nonce" TEXT,

    -- mint/burn shared props
    "sub_id" TEXT,

    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- constraints
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id"),
    FOREIGN KEY ("block_height") REFERENCES "blocks" ("block_height")
);

-- common indexes
CREATE INDEX IF NOT EXISTS idx_receipts_cursor ON "receipts" ("cursor");
CREATE INDEX IF NOT EXISTS idx_receipts_subject ON "receipts" ("subject");
CREATE INDEX IF NOT EXISTS idx_receipts_tx_id ON "receipts" ("tx_id");
CREATE INDEX IF NOT EXISTS idx_receipts_block_height ON "receipts" ("block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_type ON "receipts" ("type");

-- call/transfer shared indexes
CREATE INDEX IF NOT EXISTS idx_receipts_from_contract_id ON "receipts" ("from_contract_id");
CREATE INDEX IF NOT EXISTS idx_receipts_to_contract_id ON "receipts" ("to_contract_id");
CREATE INDEX IF NOT EXISTS idx_receipts_asset_id ON "receipts" ("asset_id");

-- contract related indexes
CREATE INDEX IF NOT EXISTS idx_receipts_contract_id ON "receipts" ("contract_id");

-- transfer_out specific indexes
CREATE INDEX IF NOT EXISTS idx_receipts_to_address ON "receipts" ("to_address");

-- message_out specific indexes
CREATE INDEX IF NOT EXISTS idx_receipts_sender_address ON "receipts" ("sender_address");
CREATE INDEX IF NOT EXISTS idx_receipts_recipient_address ON "receipts" ("recipient_address");

-- mint/burn specific indexes
CREATE INDEX IF NOT EXISTS idx_receipts_sub_id ON "receipts" ("sub_id");

-- panic specific indexes
CREATE INDEX IF NOT EXISTS idx_receipts_panic_reason ON "receipts" ("panic_reason");
CREATE INDEX IF NOT EXISTS idx_receipts_panic_instruction ON "receipts" ("panic_instruction");

-- Composite indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_receipts_type_block_height ON "receipts" ("type", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_from_contract_block_height ON "receipts" ("from_contract_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_to_contract_block_height ON "receipts" ("to_contract_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_to_address_block_height ON "receipts" ("to_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_contract_id_block_height ON "receipts" ("contract_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_sender_block_height ON "receipts" ("sender_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_recipient_block_height ON "receipts" ("recipient_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_receipts_sub_id_block_height ON "receipts" ("sub_id", "block_height");

-- Composite index for ordering
CREATE INDEX IF NOT EXISTS idx_receipts_block_height_tx_receipt ON "receipts" ("block_height", "tx_index", "receipt_index");
