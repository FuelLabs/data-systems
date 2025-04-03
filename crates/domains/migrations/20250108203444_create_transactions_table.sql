-- ------------------------------------------------------------------------------
-- Enum Types
-- ------------------------------------------------------------------------------
CREATE TYPE "transaction_type" AS ENUM (
    'script',
    'create',
    'mint',
    'upgrade',
    'upload',
    'blob'
);

CREATE TYPE "transaction_status" AS ENUM (
    'pre_confirmation_failed',
    'pre_confirmation_success',
    'failed',
    'submitted',
    'squeezed_out',
    'success'
);

-- ------------------------------------------------------------------------------
-- Main Transactions Table
-- ------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS transactions (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT NOT NULL UNIQUE,
    "block_height" BIGINT NOT NULL,
    "tx_id" TEXT UNIQUE NOT NULL,
    "tx_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{tx_index}
    -- fields matching fuel-core
    "type" transaction_type NOT NULL,
    "blob_id" TEXT,
    "bytecode_root" TEXT,
    "bytecode_witness_index" INTEGER,
    -- "input_asset_ids" TEXT[], -- (table: transaction_input_asset_ids)
    -- "input_contract" JSONB, -- (table: transaction_input_contract)
    -- "input_contracts" TEXT[], -- (table: transaction_input_contracts)
    -- "inputs" JSONB[], -- (table: transaction_inputs)
    "is_blob" BOOLEAN,
    "is_create" BOOLEAN,
    "is_mint" BOOLEAN,
    "is_script" BOOLEAN,
    "is_upgrade" BOOLEAN,
    "is_upload" BOOLEAN,
    "mint_amount" BIGINT,
    "mint_asset_id" TEXT,
    "mint_gas_price" BIGINT,
    -- "output_contract" JSONB, -- (table: transaction_output_contract)
    -- "outputs" JSONB[], -- (table: transaction_outputs)
    -- "proof_set" TEXT[], -- (table: transaction_proof_set)
    "raw_payload" TEXT NOT NULL,
    -- "receipts" JSONB[], -- (table: transaction_receipts)
    "receipts_root" TEXT,
    "salt" TEXT,
    "script" TEXT,
    "script_data" TEXT,
    "script_gas_limit" BIGINT,
    "status" transaction_status NOT NULL,
    -- "storage_slots" JSONB[], -- (table: transaction_storage_slots)
    "subsection_index" INTEGER,
    "subsections_number" INTEGER,
    "tx_pointer" BYTEA,
    "upgrade_purpose" TEXT,
    -- "witnesses" TEXT[], -- (table: transaction_witnesses)
    -- extra fields
    "maturity" INTEGER,
    -- "policies" TEXT, -- (table: transaction_policies)
    "script_length" INTEGER,
    "script_data_length" INTEGER,
    "storage_slots_count" INTEGER,
    "proof_set_count" INTEGER,
    "witnesses_count" INTEGER,
    "inputs_count" INTEGER,
    "outputs_count" INTEGER,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for the main table
CREATE INDEX IF NOT EXISTS idx_transactions_subject ON transactions (subject);
CREATE INDEX IF NOT EXISTS idx_transactions_block_height ON transactions (block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_id ON transactions (tx_id);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_index ON transactions (tx_index);
CREATE INDEX IF NOT EXISTS idx_transactions_status ON transactions (status);
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (type);
CREATE INDEX IF NOT EXISTS idx_transactions_cursor ON transactions (cursor);
CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions (blob_id);
CREATE INDEX IF NOT EXISTS idx_transactions_is_blob ON transactions (is_blob);
CREATE INDEX IF NOT EXISTS idx_transactions_is_create ON transactions (is_create);
CREATE INDEX IF NOT EXISTS idx_transactions_is_mint ON transactions (is_mint);
CREATE INDEX IF NOT EXISTS idx_transactions_is_script ON transactions (is_script);
CREATE INDEX IF NOT EXISTS idx_transactions_is_upgrade ON transactions (is_upgrade);
CREATE INDEX IF NOT EXISTS idx_transactions_is_upload ON transactions (is_upload);
CREATE INDEX IF NOT EXISTS idx_transactions_inputs_count ON transactions (inputs_count);
CREATE INDEX IF NOT EXISTS idx_transactions_outputs_count ON transactions (outputs_count);
CREATE INDEX IF NOT EXISTS idx_transactions_script ON transactions (script);
CREATE INDEX IF NOT EXISTS idx_transactions_block_time ON transactions (block_time);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at);

-- Composite indexes
CREATE INDEX IF NOT EXISTS idx_transactions_status_block_height ON transactions (status, block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_type_block_height ON transactions (type, block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_ordering ON transactions (block_height, tx_index);

-- ------------------------------------------------------------------------------
-- Input Asset IDs Table (Vec<asset_id>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_input_asset_ids" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "asset_id" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_input_asset_ids_tx_id ON transaction_input_asset_ids (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_asset_ids_block_height ON transaction_input_asset_ids (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_input_asset_ids_asset_id ON transaction_input_asset_ids (asset_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_asset_ids_block_time ON transaction_input_asset_ids (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_input_asset_ids_created_at ON transaction_input_asset_ids (created_at);

-- ------------------------------------------------------------------------------
-- Input Contract Table (Option<InputContract>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_input_contract" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "balance_root" TEXT NOT NULL,
    "contract_id" TEXT NOT NULL,
    "state_root" TEXT NOT NULL,
    "tx_pointer" BYTEA NOT NULL,
    "utxo_id" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_tx_id ON transaction_input_contract (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_block_height ON transaction_input_contract (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_contract_id ON transaction_input_contract (contract_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_utxo_id ON transaction_input_contract (utxo_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_block_time ON transaction_input_contract (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contract_created_at ON transaction_input_contract (created_at);

-- ------------------------------------------------------------------------------
-- Input Contracts Table (Option<Vec<contract_id>>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_input_contracts" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "contract_id" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_input_contracts_tx_id ON transaction_input_contracts (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contracts_contract_id ON transaction_input_contracts (contract_id);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contracts_block_height ON transaction_input_contracts (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contracts_block_time ON transaction_input_contracts (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_input_contracts_created_at ON transaction_input_contracts (created_at);

-- ------------------------------------------------------------------------------
-- Output Contract Table (Option<OutputContract>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_output_contract" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "balance_root" TEXT NOT NULL,
    "input_index" INTEGER NOT NULL,
    "state_root" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_output_contract_tx_id ON transaction_output_contract (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_output_contract_block_height ON transaction_output_contract (block_height);

-- ------------------------------------------------------------------------------
-- Storage Slots Table (Vec<StorageSlot>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_storage_slots" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "key" TEXT NOT NULL,
    "value" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_tx_id ON transaction_storage_slots (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_key ON transaction_storage_slots (key);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_block_height ON transaction_storage_slots (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_block_time ON transaction_storage_slots (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_storage_slots_created_at ON transaction_storage_slots (created_at);

-- ------------------------------------------------------------------------------
-- Witnesses Table (Vec<HexData>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_witnesses" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "witness_data" TEXT NOT NULL,
    "witness_data_length" INTEGER NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_tx_id ON transaction_witnesses (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_block_height ON transaction_witnesses (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_block_time ON transaction_witnesses (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_witnesses_created_at ON transaction_witnesses (created_at);

-- ------------------------------------------------------------------------------
-- Proof Set Table (Vec<Bytes32>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_proof_set" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "proof_hash" TEXT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_tx_id ON transaction_proof_set (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_proof_hash ON transaction_proof_set (proof_hash);
CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_block_height ON transaction_proof_set (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_block_time ON transaction_proof_set (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_proof_set_created_at ON transaction_proof_set (created_at);

-- ------------------------------------------------------------------------------
-- Policies Table (Option<PolicyWrapper>)
-- ------------------------------------------------------------------------------
CREATE TABLE "transaction_policies" (
    "id" SERIAL PRIMARY KEY,
    "tx_id" TEXT NOT NULL,
    "block_height" BIGINT NOT NULL,
    -- props
    "tip" BIGINT,
    "maturity" INTEGER,
    "witness_limit" BIGINT,
    "max_fee" BIGINT,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    FOREIGN KEY ("tx_id") REFERENCES "transactions" ("tx_id")
);

CREATE INDEX IF NOT EXISTS idx_transaction_policies_tx_id ON transaction_policies (tx_id);
CREATE INDEX IF NOT EXISTS idx_transaction_policies_block_height ON transaction_policies (block_height);
CREATE INDEX IF NOT EXISTS idx_transaction_policies_block_time ON transaction_policies (block_time);
CREATE INDEX IF NOT EXISTS idx_transaction_policies_created_at ON transaction_policies (created_at);
