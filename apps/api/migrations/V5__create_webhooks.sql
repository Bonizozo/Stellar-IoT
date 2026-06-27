CREATE TABLE webhooks (
    id         TEXT      PRIMARY KEY,
    owner      TEXT      NOT NULL,
    url        TEXT      NOT NULL,
    events     TEXT[]    NOT NULL,
    secret     TEXT      NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_webhooks_owner ON webhooks(owner);
