CREATE TABLE guilds (
    id SERIAL NOT NULL PRIMARY KEY,
    guild_id VARCHAR NOT NULL UNIQUE,
    admin_role VARCHAR,
    rules TEXT NOT NULL DEFAULT '',
    rules_channel_id VARCHAR,
    log_channel_id VARCHAR,
    rules_message_id VARCHAR,
    reaction_ok VARCHAR NOT NULL DEFAULT '✅',
    reaction_reject VARCHAR NOT NULL DEFAULT '❌', 
    active BOOLEAN NOT NULL DEFAULT FALSE,
    strict BOOLEAN NOT NULL DEFAULT FALSE
);