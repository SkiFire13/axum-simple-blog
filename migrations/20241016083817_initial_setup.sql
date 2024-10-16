-- Add migration script here

CREATE TABLE IF NOT EXISTS blogposts (
    id UUID PRIMARY KEY,
    date DATE NOT NULL,
    text TEXT NOT NULL,
    image TEXT,
    user TEXT NOT NULL,
    avatar TEXT
);
