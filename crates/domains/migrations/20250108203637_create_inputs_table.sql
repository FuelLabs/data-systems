CREATE TYPE "input_type" AS ENUM ('contract', 'coin', 'message');

-- ------------------------------------------------------------------------------
-- Inputs table
-- ------------------------------------------------------------------------------
CREATE TABLE "inputs" (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT UNIQUE NOT NULL,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_index" INTEGER NOT NULL,
    "input_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}-{input_index}
    -- common props
    "type" input_type NOT NULL,
    "utxo_id" TEXT,
    -- coin specific props
    "amount" BIGINT,
    "asset_id" TEXT,
    "owner_id" TEXT,
    -- contract specific props
    "balance_root" TEXT,
    "contract_id" TEXT,
    "state_root" TEXT,
    "tx_pointer" BYTEA,
    -- message specific props
    "sender_address" TEXT,
    "recipient_address" TEXT,
    "nonce" TEXT,
    "data" TEXT,
    "data_length" INTEGER,
    -- predicate related props (shared between coin and message)
    "witness_index" INTEGER,
    "predicate_gas_used" BIGINT,
    "predicate" TEXT,
    "predicate_data" TEXT,
    "predicate_length" INTEGER,
    "predicate_data_length" INTEGER,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    -- constraints
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id"),
    FOREIGN KEY ("block_height") REFERENCES "blocks" ("block_height")
);

-- common indexes
CREATE INDEX IF NOT EXISTS idx_inputs_cursor ON "inputs" ("cursor");
CREATE INDEX IF NOT EXISTS idx_inputs_subject ON "inputs" ("subject");
CREATE INDEX IF NOT EXISTS idx_inputs_tx_id ON "inputs" ("tx_id");
CREATE INDEX IF NOT EXISTS idx_inputs_block_height ON "inputs" ("block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_type ON "inputs" ("type");
CREATE INDEX IF NOT EXISTS idx_inputs_utxo_id ON "inputs" ("utxo_id");

-- coin specific indexes
CREATE INDEX IF NOT EXISTS idx_inputs_asset_id ON "inputs" ("asset_id");
CREATE INDEX IF NOT EXISTS idx_inputs_owner_id ON "inputs" ("owner_id");

-- contract specific index
CREATE INDEX IF NOT EXISTS idx_inputs_contract_id ON "inputs" ("contract_id");

-- message specific indexes
CREATE INDEX IF NOT EXISTS idx_inputs_sender_address ON "inputs" ("sender_address");
CREATE INDEX IF NOT EXISTS idx_inputs_recipient_address ON "inputs" ("recipient_address");
CREATE INDEX IF NOT EXISTS idx_inputs_nonce ON "inputs" ("nonce");

-- Composite indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_inputs_type_block_height ON "inputs" ("type", "block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_contract_id_block_height ON "inputs" ("contract_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_sender_address_block_height ON "inputs" ("sender_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_recipient_address_block_height ON "inputs" ("recipient_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_owner_id_block_height ON "inputs" ("owner_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_inputs_asset_id_block_height ON "inputs" ("asset_id", "block_height");

-- Composite index for ordering
CREATE INDEX IF NOT EXISTS idx_inputs_block_height_tx_input_index ON "inputs" ("block_height", "tx_index", "input_index");
