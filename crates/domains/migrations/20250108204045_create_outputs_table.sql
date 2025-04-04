CREATE TYPE "output_type" AS ENUM ('coin', 'contract', 'change', 'variable', 'contract_created');

-- ------------------------------------------------------------------------------
-- Outputs table
-- ------------------------------------------------------------------------------
CREATE TABLE "outputs" (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT UNIQUE NOT NULL,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_index" INTEGER NOT NULL,
    "output_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}-{output_index}
    -- common props
    "type" output_type NOT NULL,

    -- coin/change/variable shared props
    "amount" BIGINT,
    "asset_id" TEXT,
    "to_address" TEXT,      -- Maps to 'to' in types

    -- contract/contract_created shared props
    "state_root" TEXT,

    -- contract specific props
    "balance_root" TEXT,
    "input_index" INTEGER,

    -- contract_created specific props
    "contract_id" TEXT,     -- Maps to 'contract' in subjects

    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- constraints
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id"),
    FOREIGN KEY ("block_height") REFERENCES "blocks" ("block_height")
);

-- common indexes
CREATE INDEX IF NOT EXISTS idx_outputs_cursor ON "outputs" ("cursor");
CREATE INDEX IF NOT EXISTS idx_outputs_subject ON "outputs" ("subject");
CREATE INDEX IF NOT EXISTS idx_outputs_tx_id ON "outputs" ("tx_id");
CREATE INDEX IF NOT EXISTS idx_outputs_block_height ON "outputs" ("block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_type ON "outputs" ("type");

-- coin/change/variable specific indexes
CREATE INDEX IF NOT EXISTS idx_outputs_asset_id ON "outputs" ("asset_id");
CREATE INDEX IF NOT EXISTS idx_outputs_to_address ON "outputs" ("to_address");

-- contract specific indexes
CREATE INDEX IF NOT EXISTS idx_outputs_balance_root ON "outputs" ("balance_root");
CREATE INDEX IF NOT EXISTS idx_outputs_input_index ON "outputs" ("input_index");

-- contract/contract_created shared indexes
CREATE INDEX IF NOT EXISTS idx_outputs_state_root ON "outputs" ("state_root");
CREATE INDEX IF NOT EXISTS idx_outputs_contract_id ON "outputs" ("contract_id");

-- Composite indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_outputs_type_block_height ON "outputs" ("type", "block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_to_address_block_height ON "outputs" ("to_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_asset_id_block_height ON "outputs" ("asset_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_contract_id_block_height ON "outputs" ("contract_id", "block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_balance_root_block_height ON "outputs" ("balance_root", "block_height");
CREATE INDEX IF NOT EXISTS idx_outputs_state_root_block_height ON "outputs" ("state_root", "block_height");

-- Composite index for ordering
CREATE INDEX IF NOT EXISTS idx_outputs_block_height_tx_output_index ON "outputs" ("block_height", "tx_index", "output_index");
