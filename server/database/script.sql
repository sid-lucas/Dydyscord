CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) NOT NULL UNIQUE
);
INSERT INTO users (username)
VALUES
    ('alice'),
    ('bob'),
    ('charlie'),
    ('diana');