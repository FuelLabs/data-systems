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
    "reason" JSONB,               -- panic specific: stores PanicInstruction {reason, instruction}
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
CREATE INDEX ON "receipts" ("cursor");
CREATE INDEX ON "receipts" ("subject");
CREATE INDEX ON "receipts" ("tx_id");
CREATE INDEX ON "receipts" ("block_height");
CREATE INDEX ON "receipts" ("type");

-- call/transfer shared indexes
CREATE INDEX ON "receipts" ("from_contract_id");
CREATE INDEX ON "receipts" ("to_contract_id");
CREATE INDEX ON "receipts" ("asset_id");

-- contract related indexes
CREATE INDEX ON "receipts" ("contract_id");

-- transfer_out specific indexes
CREATE INDEX ON "receipts" ("to_address");

-- message_out specific indexes
CREATE INDEX ON "receipts" ("sender_address");
CREATE INDEX ON "receipts" ("recipient_address");

-- mint/burn specific indexes
CREATE INDEX ON "receipts" ("sub_id");

-- panic specific indexes
CREATE INDEX ON "receipts" USING GIN ("reason");

-- Composite indexes for efficient querying
CREATE INDEX ON "receipts" ("type", "block_height");
CREATE INDEX ON "receipts" ("from_contract_id", "block_height");
CREATE INDEX ON "receipts" ("to_contract_id", "block_height");
CREATE INDEX ON "receipts" ("to_address", "block_height");
CREATE INDEX ON "receipts" ("contract_id", "block_height");
CREATE INDEX ON "receipts" ("sender_address", "block_height");
CREATE INDEX ON "receipts" ("recipient_address", "block_height");
CREATE INDEX ON "receipts" ("sub_id", "block_height");

-- Composite index for ordering
CREATE INDEX ON "receipts" ("block_height", "tx_index", "receipt_index");
