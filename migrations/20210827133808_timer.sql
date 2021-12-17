-- Add migration script here
CREATE TABLE IF NOT EXISTS timers (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT,
    recurring BOOLEAN NOT NULL DEFAULT 'f',
    raid_lead VARCHAR,
    time TEXT NOT NULL,
    Channel NUMERIC(20) NOT NULL,
    uuid uuid NOT NULL
)
