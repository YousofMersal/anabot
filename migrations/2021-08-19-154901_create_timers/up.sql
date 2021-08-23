-- Your SQL goes here
CREATE TABLE timers (
    id SERIAL PRIMARY KEY,
    title VARCHAR NOT NULL,
    body TEXT,
    recurring BOOLEAN NOT NULL DEFAULT 'f',
    raid_lead VARCHAR,
    time TEXT NOT NULL
)
