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
CREATE INDEX ON "inputs" ("cursor");
CREATE INDEX ON "inputs" ("subject");
CREATE INDEX ON "inputs" ("tx_id");
CREATE INDEX ON "inputs" ("block_height");
CREATE INDEX ON "inputs" ("type");
CREATE INDEX ON "inputs" ("utxo_id");

-- coin specific indexes
CREATE INDEX ON "inputs" ("asset_id");
CREATE INDEX ON "inputs" ("owner_id");

-- contract specific index
CREATE INDEX ON "inputs" ("contract_id");

-- message specific indexes
CREATE INDEX ON "inputs" ("sender_address");
CREATE INDEX ON "inputs" ("recipient_address");
CREATE INDEX ON "inputs" ("nonce");

-- shared indexes
CREATE INDEX ON "inputs" ("predicate");
