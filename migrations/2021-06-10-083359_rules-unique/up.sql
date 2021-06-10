ALTER TABLE rules
ADD CONSTRAINT uc_rule_by_guild UNIQUE (guild_id,name);
