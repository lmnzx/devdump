-- Create Users Table
CREATE TABLE users (
    id uuid PRIMARY KEY,
    email TEXT UNIQUE NOT NULL,
    created_on TIMESTAMPTZ NOT NULL,
    last_login TIMESTAMPTZ,
    status TEXT NULL
);
