CREATE TABLE devices (
    id          TEXT        PRIMARY KEY,
    owner       TEXT        NOT NULL,
    name        TEXT        NOT NULL,
    type        TEXT        NOT NULL,
    price       NUMERIC(18,7) NOT NULL,
    status      TEXT        NOT NULL DEFAULT 'active',
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
