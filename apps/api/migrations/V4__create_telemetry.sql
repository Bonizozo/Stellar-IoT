CREATE TABLE telemetry (
    device_id TEXT        NOT NULL REFERENCES devices(id),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    data      JSONB       NOT NULL,
    PRIMARY KEY (device_id, timestamp)
);

CREATE INDEX idx_telemetry_device_id ON telemetry(device_id);
CREATE INDEX idx_telemetry_data      ON telemetry USING gin(data);
