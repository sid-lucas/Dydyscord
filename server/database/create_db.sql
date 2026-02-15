-- Create database and schema for OPAQUE user storage
-- Replace dydyscord with your desired DB name if needed.

-- CREATE DATABASE dydyscord;

-- \connect dydyscord;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";   

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    -- Privacy-preserving lookup: HMAC(pepper, normalized_username)
    login_lookup BYTEA NOT NULL UNIQUE,
    -- OPAQUE registration record
    opaque_record BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL,
    CONSTRAINT fk_devices_user
        FOREIGN KEY (user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_devices_user_id ON devices(user_id);

-- Friend requests (pending/accepted/declined/expired)
CREATE TABLE IF NOT EXISTS friend_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_user_id UUID NOT NULL,
    to_user_id   UUID NOT NULL,
    status       TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'accepted', 'declined', 'expired')),
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (from_user_id <> to_user_id),
    CONSTRAINT fk_friend_requests_from_user
        FOREIGN KEY (from_user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_friend_requests_to_user
        FOREIGN KEY (to_user_id)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT uq_friend_requests_from_to UNIQUE (from_user_id, to_user_id)
);

CREATE INDEX IF NOT EXISTS idx_friend_requests_to_user_id
    ON friend_requests (to_user_id);

CREATE INDEX IF NOT EXISTS idx_friend_requests_from_user_id
    ON friend_requests (from_user_id);

-- Final friend relations (1 row per relation)
CREATE TABLE IF NOT EXISTS friend_edges (
    user_id_low  UUID NOT NULL,
    user_id_high UUID NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CHECK (user_id_low <> user_id_high),
    CHECK (user_id_low < user_id_high),
    CONSTRAINT fk_friend_edges_low
        FOREIGN KEY (user_id_low)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT fk_friend_edges_high
        FOREIGN KEY (user_id_high)
        REFERENCES users(id)
        ON DELETE CASCADE,
    CONSTRAINT pk_friend_edges PRIMARY KEY (user_id_low, user_id_high)
);

CREATE INDEX IF NOT EXISTS idx_friend_edges_low
    ON friend_edges (user_id_low);

CREATE INDEX IF NOT EXISTS idx_friend_edges_high
    ON friend_edges (user_id_high);