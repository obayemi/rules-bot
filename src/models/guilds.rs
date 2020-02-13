use std::fmt;
use log::{debug,info};
use crate::schema::guilds;
use diesel::pg::PgConnection;
use diesel::prelude::*;

#[derive(Queryable,Identifiable,Debug)]
pub struct Guild {
    id: i32,
    guild_id: String,
    pub admin_role: Option<String>,
    rules: String,
    rules_channel_id: Option<String>,
    log_channel_id: Option<String>,
    rules_message_id: Option<String>,
    reaction_ok: String,
    reaction_reject: String,
    active: bool,
    strict: bool,
}

#[derive(Queryable,Debug)]
pub struct ActiveGuild {
    id: i32,
    pub guild_id: String,
    pub admin_role: Option<String>,
    pub rules: String,
    pub rules_channel_id: String,
    pub rules_message_id: String,
    pub log_channel_id: Option<String>,
    pub reaction_ok: String,
    pub reaction_reject: String,
    pub strict: bool,
}

#[derive(AsChangeset)]
#[table_name="guilds"]
pub struct RulesMessageUpdate {
    pub rules_message_id: String,
    pub rules_channel_id: String,
}
impl Into<GuildUpdate> for RulesMessageUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::RulesMessageUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name="guilds"]
pub struct RulesContentUpdate {
    pub rules: String,
}
impl Into<GuildUpdate> for RulesContentUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::RulesContentUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name="guilds"]
pub struct ModeratorGroupUpdate {
    pub admin_role: String,
}
impl Into<GuildUpdate> for ModeratorGroupUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::ModeratorGroupUpdate(self)
    }
}

pub enum GuildUpdate {
    RulesMessageUpdate(RulesMessageUpdate),
    RulesContentUpdate(RulesContentUpdate),
    ModeratorGroupUpdate(ModeratorGroupUpdate),
}

impl Into<ActiveGuild> for Guild {
    fn into(self) -> ActiveGuild {
        ActiveGuild {
            id: self.id,
            guild_id: self.guild_id,
            admin_role: self.admin_role,
            rules: self.rules,
            rules_channel_id: self.rules_channel_id.unwrap(),
            rules_message_id: self.rules_message_id.unwrap(),
            log_channel_id: self.log_channel_id,
            reaction_ok: self.reaction_ok,
            reaction_reject: self.reaction_reject,
            strict: self.strict
        }
    }
}

use crate::errors::BotError;

impl Guild {
    pub fn new(guild_id: String) -> NewGuild {
        NewGuild{guild_id}
    }

    pub fn active_from_guild_id(connection: &PgConnection, guild_id: &str) -> Result<ActiveGuild,BotError> {
        info!("fetching active guild: {}", guild_id);
        Ok(guilds::table
            .filter(guilds::guild_id.eq(guild_id))
            .filter(guilds::active.eq(true))
            .filter(guilds::rules_channel_id.is_not_null())
            .get_result::<Guild>(connection)?.into())
    }

    pub fn from_guild_id(connection: &PgConnection, guild_id: &str) -> Result<Self,BotError> {
        Ok(guilds::table.filter(guilds::guild_id.eq(guild_id))
            .get_result::<Self>(connection)?)
    }

    pub fn update<Update: Into<GuildUpdate>>(
        &self,
        connection: &PgConnection,
        change: Update,
    ) -> Result<Self,BotError> {
        Ok(match change.into() {
            GuildUpdate::RulesMessageUpdate(u) => {
                diesel::update(self)
                .set(u)
                .get_result(connection)
            },
            GuildUpdate::RulesContentUpdate(u) => {
                diesel::update(self)
                .set(u)
                .get_result(connection)
            },
            GuildUpdate::ModeratorGroupUpdate(u) => {
                diesel::update(self)
                .set(u)
                .get_result(connection)
            }
        }?)
    }
}

#[derive(Insertable)]
#[table_name="guilds"]
pub struct NewGuild {
    guild_id: String,
}
impl NewGuild {
    pub fn insert(&self, connection: &PgConnection) {
        debug!("inserting: {}", self.guild_id);
        diesel::insert_into(guilds::table)
            .values(self)
            .on_conflict(guilds::guild_id)
            .do_nothing()
            .execute(connection)
            .unwrap();
    }
}

impl fmt::Display for Guild {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.guild_id, if self.active {"active"} else {"inactive"})
    }
}