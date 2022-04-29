-- Add migration script here
CREATE TABLE IF NOT EXISTS timers (
    id INTEGER NOT NULL UNIQUE,
    title TEXT NOT NULL,
    body TEXT,
    recurring BOOLEAN NOT NULL DEFAULT FALSE,
    raid_lead TEXT,
    time TEXT NOT NULL,
    channel NUMERIC NOT NULL,
    uuid uuid NOT NULL,
    PRIMARY KEY("id" AUTOINCREMENT)
)
