CREATE TYPE consensus_type AS ENUM ('genesis', 'poa_consensus');

-- Create records table
CREATE TABLE IF NOT EXISTS blocks (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT NOT NULL UNIQUE,
    "block_height" BIGINT NOT NULL UNIQUE,
    "block_da_height" BIGINT NOT NULL,
    -- other props
    "version" VARCHAR(10) NOT NULL,
    "producer_address" TEXT NOT NULL,
    -- block header
    "header_application_hash" TEXT NOT NULL,
    "header_consensus_parameters_version" INTEGER NOT NULL,
    "header_da_height" BIGINT NOT NULL,
    "header_event_inbox_root" TEXT NOT NULL,
    "header_message_outbox_root" TEXT NOT NULL,
    "header_message_receipt_count" INTEGER NOT NULL,
    "header_prev_root" TEXT NOT NULL,
    "header_state_transition_bytecode_version" INTEGER NOT NULL,
    "header_time" TIMESTAMP WITH TIME ZONE NOT NULL,
    "header_transactions_count" SMALLINT NOT NULL,
    "header_transactions_root" TEXT NOT NULL,
    "header_version" TEXT NOT NULL,
    -- block consensus
    "consensus_chain_config_hash" TEXT,
    "consensus_coins_root" TEXT,
    "consensus_type" consensus_type NOT NULL,
    "consensus_contracts_root" TEXT,
    "consensus_messages_root" TEXT,
    "consensus_signature" TEXT,
    "consensus_transactions_root" TEXT,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW (),
    "block_propagation_ms" INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_blocks_subject ON blocks (subject);
CREATE INDEX IF NOT EXISTS idx_blocks_producer_address ON blocks (producer_address);
CREATE INDEX IF NOT EXISTS idx_blocks_block_da_height ON blocks (block_da_height);
CREATE INDEX IF NOT EXISTS idx_blocks_block_height ON blocks (block_height);
CREATE INDEX IF NOT EXISTS idx_blocks_header_time ON blocks (header_time);
CREATE INDEX IF NOT EXISTS idx_blocks_header_version ON blocks (header_version);
CREATE INDEX IF NOT EXISTS idx_blocks_consensus_type ON blocks (consensus_type);
CREATE INDEX IF NOT EXISTS idx_blocks_block_time ON blocks (block_time);
CREATE INDEX IF NOT EXISTS idx_blocks_created_at ON blocks (created_at);
