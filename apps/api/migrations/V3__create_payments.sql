CREATE TABLE payments (
    id           TEXT        PRIMARY KEY,
    tx_hash      TEXT        NOT NULL UNIQUE,
    amount       NUMERIC(18,7) NOT NULL,
    device_id    TEXT        NOT NULL REFERENCES devices(id),
    user_address TEXT        NOT NULL,
    verified_at  TIMESTAMPTZ
);

CREATE INDEX idx_payments_device_id    ON payments(device_id);
CREATE INDEX idx_payments_user_address ON payments(user_address);
