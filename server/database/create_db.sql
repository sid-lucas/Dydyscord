-- Create database and schema for OPAQUE user storage
-- Replace dydyscord with your desired DB name if needed.

-- CREATE DATABASE dydyscord;

-- \connect dydyscord;

CREATE TABLE IF NOT EXISTS users (
    -- UUIDv4
    id UUID PRIMARY KEY,
    -- Privacy-preserving lookup: HMAC(pepper, normalized_username)
    login_lookup BYTEA NOT NULL UNIQUE,
    -- OPAQUE registration record
    opaque_record BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
