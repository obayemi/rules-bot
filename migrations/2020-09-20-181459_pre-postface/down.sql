-- This file should undo anything in `up.sql`
ALTER TABLE guilds
    DROP COLUMN preface,
    DROP COLUMN postface;

