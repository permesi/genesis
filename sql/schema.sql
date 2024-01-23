-- psql -U <user> -d genesis -f schema.sql

-- Create the table for the tokens
DROP TABLE IF EXISTS tokens;

CREATE TABLE tokens (
    token char(26) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (token)
);
