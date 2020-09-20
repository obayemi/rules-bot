CREATE TABLE rules (
    id SERIAL NOT NULL PRIMARY KEY,
    guild_id integer REFERENCES guilds NOT NULL,
    name VARCHAR(30) NOT NULL,
    rule TEXT NOT NULL,
    extra TEXT NOT NULL
);
