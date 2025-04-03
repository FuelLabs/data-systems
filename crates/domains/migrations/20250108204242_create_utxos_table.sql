CREATE TYPE "utxo_type" AS ENUM (
    'input_contract',
    'input_coin',
    'output_coin',
    'output_variable',
    'output_change'
);

CREATE TYPE "utxo_status" AS ENUM (
    'unspent',
    'spent'
);

-- ------------------------------------------------------------------------------
-- UTXOs table
-- ------------------------------------------------------------------------------
CREATE TABLE "utxos" (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT UNIQUE NOT NULL,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_index" INTEGER NOT NULL,
    "input_index" INTEGER,
    "output_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}-{input_index}-{output_index?}
    "utxo_id" TEXT NOT NULL UNIQUE,
    -- props
    "type" utxo_type NOT NULL,
    "status" utxo_status NOT NULL,
    "asset_id" TEXT,
    "amount" BIGINT,
    "from_address" TEXT,
    "to_address" TEXT,
    "nonce" TEXT,
    "contract_id" TEXT,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- constraints
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id"),
    FOREIGN KEY ("block_height") REFERENCES "blocks" ("block_height")
);

-- common indexes
CREATE INDEX IF NOT EXISTS idx_utxos_subject ON "utxos" ("subject");
CREATE INDEX IF NOT EXISTS idx_utxos_tx_id ON "utxos" ("tx_id");
CREATE INDEX IF NOT EXISTS idx_utxos_block_height ON "utxos" ("block_height");
CREATE INDEX IF NOT EXISTS idx_utxos_cursor ON "utxos" ("cursor");
CREATE INDEX IF NOT EXISTS idx_utxos_type ON "utxos" ("type");
CREATE INDEX IF NOT EXISTS idx_utxos_status ON "utxos" ("status");
CREATE INDEX IF NOT EXISTS idx_utxos_amount ON "utxos" ("amount");
CREATE INDEX IF NOT EXISTS idx_utxos_from_address ON "utxos" ("from_address");
CREATE INDEX IF NOT EXISTS idx_utxos_to_address ON "utxos" ("to_address");
CREATE INDEX IF NOT EXISTS idx_utxos_utxo_id ON "utxos" ("utxo_id");
CREATE INDEX IF NOT EXISTS idx_utxos_contract_id ON "utxos" ("contract_id");

-- Composite indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_utxos_type_block_height ON "utxos" ("type", "block_height");
CREATE INDEX IF NOT EXISTS idx_utxos_to_address_block_height ON "utxos" ("to_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_utxos_from_address_block_height ON "utxos" ("from_address", "block_height");
CREATE INDEX IF NOT EXISTS idx_utxos_contract_id_block_height ON "utxos" ("contract_id", "block_height");

-- Composite indexes for ordering
CREATE INDEX IF NOT EXISTS idx_utxos_block_height_tx_input ON "utxos" ("block_height", "tx_index", "input_index");
CREATE INDEX IF NOT EXISTS idx_utxos_block_height_tx_input_output ON "utxos" ("block_height", "tx_index", "input_index", "output_index");
