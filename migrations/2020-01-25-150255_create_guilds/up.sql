CREATE TABLE guilds (
    id SERIAL NOT NULL PRIMARY KEY,
    guild_id BIGINT NOT NULL UNIQUE,
    admin_role BIGINT,
    rules TEXT NOT NULL DEFAULT '',
    rules_channel_id BIGINT,
    log_channel_id BIGINT,
    rules_message_id BIGINT,
    reaction_ok VARCHAR NOT NULL DEFAULT '✅',
    reaction_reject VARCHAR NOT NULL DEFAULT '❌', 
    active BOOLEAN NOT NULL DEFAULT FALSE,
    strict BOOLEAN NOT NULL DEFAULT FALSE
);
