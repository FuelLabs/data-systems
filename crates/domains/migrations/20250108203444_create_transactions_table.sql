-- ------------------------------------------------------------------------------
-- Enum Types
-- ------------------------------------------------------------------------------
CREATE TYPE "transaction_type" AS ENUM (
    'SCRIPT',
    'CREATE',
    'MINT',
    'UPGRADE',
    'UPLOAD',
    'BLOB'
);

CREATE TYPE "transaction_status" AS ENUM (
    'FAILED',
    'SUBMITTED',
    'SQUEEZED_OUT',
    'SUCCESS',
    'NONE'
);

-- ------------------------------------------------------------------------------
-- Main Transactions Table
-- ------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_id" TEXT UNIQUE NOT NULL,
    "tx_index" INTEGER NOT NULL,
    -- cursor
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}
    -- fields matching fuel-core
    "type" transaction_type NOT NULL,
    "script_gas_limit" BIGINT,
    "mint_amount" BIGINT,
    "mint_asset_id" TEXT,
    "mint_gas_price" BIGINT,
    "receipts_root" TEXT,
    "tx_status" transaction_status NOT NULL,
    "script" TEXT,
    "script_data" TEXT,
    "salt" TEXT,
    "bytecode_witness_index" INTEGER,
    "bytecode_root" TEXT,
    "subsection_index" INTEGER,
    "subsections_number" INTEGER,
    "upgrade_purpose" TEXT,
    "blob_id" TEXT,
    -- extra fields (not in fuel-core)
    "maturity" INTEGER,
    "policies" TEXT,
    "script_length" BIGINT,
    "script_data_length" BIGINT,
    "storage_slots_count" BIGINT,
    "proof_set_count" INTEGER,
    "witnesses_count" INTEGER,
    "inputs_count" INTEGER,
    "outputs_count" INTEGER,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    "published_at" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_transactions_subject ON transactions (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_id ON transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_index ON transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status ON transactions (tx_status);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (type);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_published_at ON transactions (published_at);
CREATE INDEX IF NOT EXISTS idx_transactions_cursor ON transactions (cursor);
CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions (blob_id);
CREATE INDEX IF NOT EXISTS idx_transactions_script ON transactions (script);
CREATE INDEX IF NOT EXISTS idx_transactions_policies ON transactions (policies);

-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status_block_height ON transactions (tx_status, block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_type_block_height ON transactions (type, block_height);

-- Composite index for ordering by (block_height, tx_index)
CREATE INDEX IF NOT EXISTS idx_transactions_ordering ON transactions (block_height, tx_index);

-- ------------------------------------------------------------------------------
-- Storage Slots Table
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_storage_slots" (
    "_id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "key" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_tx_id ON transaction_storage_slots (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_key ON transaction_storage_slots (key);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_value ON transaction_storage_slots (value);

-- ------------------------------------------------------------------------------
-- Witnesses Table
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_witnesses" (
    "_id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "witness_data" TEXT NOT NULL,
    "witness_data_length" INTEGER NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_tx_id ON transaction_witnesses (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_witness_data ON transaction_witnesses (witness_data);

-- ------------------------------------------------------------------------------
-- Proof Set Table
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_proof_set" (
    "_id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "proof_hash" TEXT NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_tx_id ON transaction_proof_set (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_proof_hash ON transaction_proof_set (proof_hash);
