-- ------------------------------------------------------------------------------
-- Enum Types
-- ------------------------------------------------------------------------------
CREATE TYPE "message_type" AS ENUM (
    'imported',
    'consumed'
);

-- ------------------------------------------------------------------------------
-- Main Messages Table
-- ------------------------------------------------------------------------------
CREATE TABLE IF NOT EXISTS messages (
    "id" SERIAL PRIMARY KEY,
    "value" BYTEA NOT NULL,
    -- uniques
    "subject" TEXT UNIQUE NOT NULL,
    "block_height" BIGINT NOT NULL,
    "message_index" INTEGER NOT NULL,
    "cursor" TEXT NOT NULL, -- {block_height}-{message_index}
    -- fields matching fuel-core
    "type" message_type NOT NULL,
    "sender" TEXT NOT NULL,
    "recipient" TEXT NOT NULL,
    "nonce" TEXT NOT NULL,
    "amount" BIGINT NOT NULL,
    "data" TEXT NOT NULL,
    "da_height" BIGINT NOT NULL,
    -- timestamps
    "block_time" TIMESTAMPTZ NOT NULL,
    "created_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    "updated_at" TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- constraints
    FOREIGN KEY ("block_height") REFERENCES "blocks" ("block_height")
);

-- ------------------------------------------------------------------------------
-- Indexes
-- ------------------------------------------------------------------------------
CREATE INDEX IF NOT EXISTS idx_messages_subject ON messages (subject);
CREATE INDEX IF NOT EXISTS idx_messages_type ON messages (type);
CREATE INDEX IF NOT EXISTS idx_messages_sender ON messages (sender);
CREATE INDEX IF NOT EXISTS idx_messages_recipient ON messages (recipient);
CREATE INDEX IF NOT EXISTS idx_messages_nonce ON messages (nonce);
CREATE INDEX IF NOT EXISTS idx_messages_da_height ON messages (da_height);
CREATE INDEX IF NOT EXISTS idx_messages_block_height ON messages (block_height);
CREATE INDEX IF NOT EXISTS idx_messages_cursor ON messages (cursor);
CREATE INDEX IF NOT EXISTS idx_messages_block_time ON messages (block_time);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages (created_at);

-- Composite indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_messages_type_block_height ON messages (type, block_height);
CREATE INDEX IF NOT EXISTS idx_messages_sender_block_height ON messages (sender, block_height);
CREATE INDEX IF NOT EXISTS idx_messages_recipient_block_height ON messages (recipient, block_height);
