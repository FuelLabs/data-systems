-- Create API scope enum type
CREATE TYPE api_scope AS ENUM ('HISTORICAL_DATA', 'LIVE_DATA', 'REST_API', 'MANAGE_API_KEYS');
CREATE TYPE api_role AS ENUM ('ADMIN', 'AMM', 'BUILDER', 'WEB_CLIENT');
CREATE TYPE api_key_status AS ENUM ('ACTIVE', 'INACTIVE', 'REVOKED', 'EXPIRED');

-- Create api_key_roles table
CREATE TABLE IF NOT EXISTS api_key_roles (
    id SERIAL PRIMARY KEY,
    name api_role NOT NULL UNIQUE,
    subscription_limit INTEGER,
    scopes api_scope[] NOT NULL DEFAULT '{}',
    rate_limit_per_minute INTEGER,
    -- The number of blocks to include in the historical data stream
    historical_limit INTEGER
);

-- Insert default roles
INSERT INTO api_key_roles (name, subscription_limit, scopes, rate_limit_per_minute, historical_limit) VALUES
    ('ADMIN', NULL, ARRAY['HISTORICAL_DATA', 'LIVE_DATA', 'REST_API', 'MANAGE_API_KEYS']::api_scope[], NULL, NULL),
    ('AMM', NULL, ARRAY['HISTORICAL_DATA', 'LIVE_DATA', 'REST_API']::api_scope[], NULL, NULL),
    ('BUILDER', 50, ARRAY['HISTORICAL_DATA', 'LIVE_DATA', 'REST_API']::api_scope[], NULL, 600),
    ('WEB_CLIENT', NULL, ARRAY['LIVE_DATA', 'REST_API']::api_scope[], 1000, NULL);

-- Create api_keys table with all required fields
CREATE TABLE IF NOT EXISTS api_keys (
    id SERIAL PRIMARY KEY,
    user_name VARCHAR NOT NULL UNIQUE,
    api_key VARCHAR NOT NULL UNIQUE,
    role_id INTEGER NOT NULL REFERENCES api_key_roles(id),
    status api_key_status NOT NULL DEFAULT 'ACTIVE'
);

-- Create indexes for better performance
CREATE INDEX IF NOT EXISTS idx_api_keys_role_id ON api_keys (role_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_status ON api_keys (status);
