-- Create API scope enum type
CREATE TYPE api_scope AS ENUM ('FULL', 'HISTORICAL_DATA', 'LIVE_DATA', 'REST_API');
CREATE TYPE api_role AS ENUM ('ADMIN', 'BUILDER', 'WEB_CLIENT');
CREATE TYPE api_key_status AS ENUM ('ACTIVE', 'INACTIVE', 'REVOKED', 'EXPIRED');

-- Create api_key_roles table
CREATE TABLE IF NOT EXISTS api_key_roles (
    id SERIAL PRIMARY KEY,
    name api_role NOT NULL UNIQUE,
    subscription_limit INTEGER,
    scopes api_scope[] NOT NULL DEFAULT '{}',
    rate_limit_per_minute INTEGER
);

-- Add unique constraint to user_name in api_keys table
ALTER TABLE api_keys
ADD CONSTRAINT unique_user_name UNIQUE (user_name);

-- Insert default roles
INSERT INTO api_key_roles (name, subscription_limit, scopes, rate_limit_per_minute) VALUES
    ('ADMIN', NULL, ARRAY['FULL']::api_scope[], NULL),
    ('BUILDER', 50, ARRAY['FULL']::api_scope[], NULL),
    ('WEB_CLIENT', NULL, ARRAY['LIVE_DATA', 'REST_API']::api_scope[], 1000);

-- Add role_id column to api_keys table as a foreign key
ALTER TABLE api_keys
ADD COLUMN role_id INTEGER;

-- Create an index on the role_id column for faster lookups
CREATE INDEX IF NOT EXISTS idx_api_keys_role_id ON api_keys (role_id);

-- Add foreign key constraint
ALTER TABLE api_keys
ADD CONSTRAINT fk_api_keys_role
FOREIGN KEY (role_id) REFERENCES api_key_roles(id);

-- Set default role (WEB_CLIENT) for existing api keys
UPDATE api_keys
SET role_id = (SELECT id FROM api_key_roles WHERE name = 'WEB_CLIENT');

-- Make role_id NOT NULL after setting defaults
ALTER TABLE api_keys
ALTER COLUMN role_id SET NOT NULL;

-- Add status column to api_keys table
ALTER TABLE api_keys
ADD COLUMN status api_key_status NOT NULL DEFAULT 'INACTIVE';

-- Create an index on the status column for faster filtering
CREATE INDEX IF NOT EXISTS idx_api_keys_status ON api_keys (status);

-- Set existing api keys to ACTIVE status
UPDATE api_keys
SET status = 'ACTIVE';

-- Change user_id to id in api_keys table
-- First, drop the primary key constraint
ALTER TABLE api_keys
DROP CONSTRAINT api_keys_pkey;

-- Rename the column
ALTER TABLE api_keys
RENAME COLUMN user_id TO id;

-- Ensure the column is SERIAL (this preserves the sequence)
-- The sequence name will automatically be updated when the column is renamed
-- but we can verify it's still connected to the column
ALTER TABLE api_keys ALTER COLUMN id SET DEFAULT nextval('api_keys_user_id_seq'::regclass);

-- Rename the sequence to match the new column name
ALTER SEQUENCE api_keys_user_id_seq RENAME TO api_keys_id_seq;

-- Add the primary key constraint back on the renamed column
ALTER TABLE api_keys
ADD CONSTRAINT api_keys_pkey PRIMARY KEY (id);
