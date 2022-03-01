-- Add up migration script here
CREATE TABLE guilds (
    guild_id BIGINT PRIMARY KEY,
    admin_role_id BIGINT,
    rules_channel_id BIGINT,
    log_channel_id BIGINT,
    rules_message_id BIGINT,
    reaction_ok VARCHAR NOT NULL DEFAULT '✅',
    reaction_reject VARCHAR NOT NULL DEFAULT '❌', 
    active BOOLEAN NOT NULL DEFAULT FALSE,
    member_role_id BIGINT,
    preface TEXT NOT NULL DEFAULT '',
    postface TEXT NOT NULL DEFAULT ''
);

CREATE TABLE rules (
    guild_id BIGINT REFERENCES guilds NOT NULL,
    name VARCHAR(30) NOT NULL,
    rule TEXT NOT NULL,
    extra TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (guild_id,name)
);