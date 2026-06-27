CREATE TABLE sessions (
    id           TEXT        PRIMARY KEY,
    device_id    TEXT        NOT NULL REFERENCES devices(id),
    user_address TEXT        NOT NULL,
    tx_hash      TEXT        NOT NULL,
    started_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at   TIMESTAMPTZ NOT NULL
);

CREATE INDEX idx_sessions_device_id    ON sessions(device_id);
CREATE INDEX idx_sessions_user_address ON sessions(user_address);
