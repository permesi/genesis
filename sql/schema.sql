-- psql -U <user> -d genesis -f schema.sql
-- Create the table for the tokens
CREATE TABLE tokens (
    id bytea NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id)
);
