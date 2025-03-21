CREATE TYPE "ConsensusType" AS ENUM ('GENESIS', 'POA_CONSENSUS');

-- Create records table
CREATE TABLE IF NOT EXISTS blocks (
    -- uniques
    "id" SERIAL PRIMARY KEY,
    "subject" TEXT NOT NULL UNIQUE,
    "block_height" BIGINT NOT NULL,
    -- messaging only
    value BYTEA NOT NULL,
    -- other props
    "version" VARCHAR(10) NOT NULL,
    "producer_address" TEXT NOT NULL,
    -- timestamps
    "created_at" TIMESTAMP WITH TIME ZONE NOT NULL,
    "published_at" TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    "block_propagation_ms" TIMESTAMPTZ NOT NULL,
    -- block header
    "header_application_hash" TEXT NOT NULL,
    "header_consensus_parameters_version" INTEGER NOT NULL,
    "header_da_height" BIGINT NOT NULL,
    "header_event_inbox_root" TEXT NOT NULL,
    "header_message_outbox_root" TEXT NOT NULL,
    "header_message_receipt_count" INTEGER NOT NULL,
    "header_prev_root" TEXT NOT NULL,
    "header_state_transition_bytecode_version" INTEGER NOT NULL,
    "header_time" BIGINT NOT NULL,
    "header_transactions_count" SMALLINT NOT NULL,
    "header_transactions_root" TEXT NOT NULL,
    "header_version" INTEGER NOT NULL,
    -- block consensus
    "consensus_chain_config_hash" TEXT,
    "consensus_chain_id" BIGINT NOT NULL,
    "consensus_coins_root" TEXT,
    "consensus_type" ConsensusType NOT NULL,
    "contracts_root" TEXT,
    "consensus_messages_root" TEXT,
    "consensus_producer" TEXT NOT NULL,
    "consensus_signature" TEXT,
    "consensus_transactions_root" TEXT,
);

CREATE INDEX IF NOT EXISTS idx_blocks_subject ON blocks (subject);
CREATE INDEX IF NOT EXISTS idx_blocks_producer_address ON blocks (producer_address);
CREATE INDEX IF NOT EXISTS idx_blocks_block_da_height ON blocks (block_da_height);
CREATE INDEX IF NOT EXISTS idx_blocks_block_height ON blocks (block_height);
CREATE INDEX IF NOT EXISTS idx_blocks_created_at ON blocks (created_at);
CREATE INDEX IF NOT EXISTS idx_blocks_published_at ON blocks (published_at);
