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

CREATE TYPE "policy_type" AS ENUM (
    'TIP',
    'WITNESS_LIMIT',
    'MATURITY',
    'MAX_FEE'
);

CREATE TYPE "transaction_account_type" AS ENUM (
    'CONTRACT',
    'ADDRESS',
    'PREDICATE',
    'SCRIPT'
);

CREATE TABLE IF NOT EXISTS transactions (
    id SERIAL PRIMARY KEY,
    subject TEXT NOT NULL UNIQUE,
    value BYTEA NOT NULL,
    "block_height" BIGINT NOT NULL,
    "tx_id" TEXT UNIQUE NOT NULL,
    "tx_index" INTEGER NOT NULL,
    -- cursor
    "cursor" TEXT UNIQUE NOT NULL, -- {block_height}-{tx_index}
    -- fields matching fuel-core
    "transaction_type" transaction_type NOT NULL,
    "script_gas_limit" BIGINT,
    "is_create" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_mint" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_script" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_upgrade" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_upload" BOOLEAN NOT NULL DEFAULT FALSE,
    "is_blob" BOOLEAN NOT NULL DEFAULT FALSE,
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
    "policy_type" INTEGER,
    "script_length" BIGINT,
    "script_data_length" BIGINT,
    "storage_slots_count" BIGINT,
    "proof_set_count" INTEGER,
    "witnesses_count" INTEGER,
    "inputs_count" INTEGER,
    "outputs_count" INTEGER,
    -- from transactions_data
    "transaction_data" JSONB NOT NULL,
    -- from transaction_storage_slots
    "transaction_storage_slots_key" TEXT[] NOT NULL DEFAULT '{}',
    "transaction_storage_slots_value" TEXT[] NOT NULL DEFAULT '{}',
    -- from transaction_witnesses
    "witness_data" TEXT[] NOT NULL DEFAULT '{}',
    "witness_data_length" INTEGER[] NOT NULL DEFAULT '{}',
    -- from transaction_proof_set
    "transaction_proof_set_proof_hash" TEXT[] NOT NULL DEFAULT '{}',
    -- from transaction_policies
    "transaction_policy_type" policy_type[] NOT NULL DEFAULT '{}',
    "transaction_policy_data" TEXT[] NOT NULL DEFAULT '{}',
    -- from transaction_accounts
    "transaction_account_address" TEXT[] NOT NULL DEFAULT '{}',
    "transaction_account_type" transaction_account_type[] NOT NULL DEFAULT '{}',
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
CREATE INDEX IF NOT EXISTS idx_transactions_type ON transactions (transaction_type);
CREATE INDEX IF NOT EXISTS idx_transactions_created_at ON transactions (created_at);
CREATE INDEX IF NOT EXISTS idx_transactions_published_at ON transactions (published_at);
CREATE INDEX IF NOT EXISTS idx_transactions_cursor ON transactions (cursor);
CREATE INDEX IF NOT EXISTS idx_transactions_blob_id ON transactions (blob_id);
CREATE INDEX IF NOT EXISTS idx_transactions_script ON transactions (script);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_storage_slots_key ON transactions (transaction_storage_slots_key);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_storage_slots_value ON transactions (transaction_storage_slots_value);
CREATE INDEX IF NOT EXISTS idx_transactions_witness_data ON transactions (witness_data);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_proof_set_proof_hash ON transactions (transaction_proof_set_proof_hash);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_policy_type ON transactions (transaction_policy_type);
CREATE INDEX IF NOT EXISTS idx_transactions_transaction_account_type ON transactions (transaction_account_type);


-- Composite indexes for filtering with "WHERE block_height >= <value>"
CREATE INDEX IF NOT EXISTS idx_transactions_tx_status_block_height ON transactions (tx_status, block_height);
CREATE INDEX IF NOT EXISTS idx_transactions_tx_type_block_height ON transactions (transaction_type, block_height);

-- Composite index for ordering by (block_height, tx_index)
CREATE INDEX IF NOT EXISTS idx_transactions_ordering ON transactions (block_height, tx_index);
