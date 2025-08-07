-- Add migration script here
CREATE TABLE users(
    user_id uuid PRIMARY KEY,
    username TEXT NOT NULL,
    password TEXT NOT NULL
);