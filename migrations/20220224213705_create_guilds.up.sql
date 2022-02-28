-- Add up migration script here
CREATE TABLE guilds (
    id SERIAL NOT NULL PRIMARY KEY,
    guild_id BIGINT NOT NULL UNIQUE,
    admin_role BIGINT,
    rules_channel_id BIGINT,
    log_channel_id BIGINT,
    rules_message_id BIGINT,
    reaction_ok VARCHAR NOT NULL DEFAULT '✅',
    reaction_reject VARCHAR NOT NULL DEFAULT '❌', 
    active BOOLEAN NOT NULL DEFAULT FALSE,
    member_role BIGINT,
    preface TEXT NOT NULL DEFAULT '',
    postface TEXT NOT NULL DEFAULT ''
);

CREATE TABLE rules (
    id SERIAL NOT NULL PRIMARY KEY,
    guild_id integer REFERENCES guilds NOT NULL,
    name VARCHAR(30) NOT NULL,
    rule TEXT NOT NULL,
    extra TEXT NOT NULL DEFAULT '',
    UNIQUE (guild_id,name)
);