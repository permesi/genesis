-- psql -U <user> -d genesis -f schema.sql

-- https://github.com/pksunkara/pgx_ulid
-- CREATE EXTENSION ulid;

-- Create the table for clients
DROP TABLE IF EXISTS clients CASCADE;
CREATE TABLE clients (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    uuid UUID DEFAULT uuid_generate_v4()
);

INSERT INTO clients (id, name, uuid) VALUES (0, 'unknown', '00000000-0000-0000-0000-000000000000');

-- Create the table for the tokens
DROP TABLE IF EXISTS tokens CASCADE;
CREATE TABLE tokens (
    id ulid NOT NULL DEFAULT gen_ulid() PRIMARY KEY,
    client_id INTEGER DEFAULT 0 REFERENCES clients(id)
);

-- Create the table for the metadata
DROP TABLE IF EXISTS metadata;
CREATE TABLE metadata (
    id ulid PRIMARY KEY REFERENCES tokens(id),
    ip_address INET,
    country CHAR(2),
    user_agent VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_metadata_country ON metadata(country);
