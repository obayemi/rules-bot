use crate::schema::guilds;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use log::{debug, info};
use std::fmt;

#[derive(Queryable, Identifiable, Debug)]
pub struct Guild {
    id: i32,
    pub guild_id: i64,
    pub admin_role: Option<i64>,
    pub rules: String,
    pub rules_channel_id: Option<i64>,
    pub log_channel_id: Option<i64>,
    pub rules_message_id: Option<i64>,
    pub reaction_ok: String,
    pub reaction_reject: String,
    pub active: bool,
    pub strict: bool,
    pub member_role: Option<i64>,
}

#[derive(Queryable, Debug)]
pub struct ActiveGuild {
    id: i32,
    pub guild_id: i64,
    pub admin_role: Option<i64>,
    pub rules: String,
    pub rules_channel_id: i64,
    pub rules_message_id: i64,
    pub log_channel_id: Option<i64>,
    pub reaction_ok: String,
    pub reaction_reject: String,
    pub strict: bool,
    pub member_role: i64,
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct RulesMessageUpdate {
    pub rules_message_id: i64,
    pub rules_channel_id: i64,
    pub rules: String,
}
impl Into<GuildUpdate> for RulesMessageUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::RulesMessageUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct RulesContentUpdate {
    pub rules: String,
}
impl Into<GuildUpdate> for RulesContentUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::RulesContentUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct ModeratorRoleUpdate {
    pub admin_role: i64,
}
impl Into<GuildUpdate> for ModeratorRoleUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::ModeratorRoleUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct MemberRoleUpdate {
    pub member_role: i64,
}
impl Into<GuildUpdate> for MemberRoleUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::MemberRoleUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct RulesChannelUpdate {
    pub rules_channel_id: i64,
}
impl Into<GuildUpdate> for RulesChannelUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::RulesChannelUpdate(self)
    }
}

#[derive(AsChangeset)]
#[table_name = "guilds"]
pub struct LogsChannelUpdate {
    pub log_channel_id: i64,
}
impl Into<GuildUpdate> for LogsChannelUpdate {
    fn into(self) -> GuildUpdate {
        GuildUpdate::LogsChannelUpdate(self)
    }
}

pub enum GuildUpdate {
    RulesMessageUpdate(RulesMessageUpdate),
    RulesContentUpdate(RulesContentUpdate),
    ModeratorRoleUpdate(ModeratorRoleUpdate),
    MemberRoleUpdate(MemberRoleUpdate),
    RulesChannelUpdate(RulesChannelUpdate),
    LogsChannelUpdate(LogsChannelUpdate),
    ClearModeratorRole,
    UnbindMessage,
    EnableBot,
    DisableBot,
}

impl Into<ActiveGuild> for Guild {
    fn into(self) -> ActiveGuild {
        ActiveGuild {
            id: self.id,
            guild_id: self.guild_id,
            admin_role: self.admin_role,
            member_role: self.member_role.unwrap(),
            rules: self.rules,
            rules_channel_id: self.rules_channel_id.unwrap(),
            rules_message_id: self.rules_message_id.unwrap(),
            log_channel_id: self.log_channel_id,
            reaction_ok: self.reaction_ok,
            reaction_reject: self.reaction_reject,
            strict: self.strict,
        }
    }
}

use crate::errors::BotError;

impl Guild {
    pub fn new(guild_id: i64) -> NewGuild {
        NewGuild { guild_id }
    }

    pub fn active_from_guild_id(
        connection: &PgConnection,
        guild_id: i64,
    ) -> Result<ActiveGuild, BotError> {
        info!("fetching active guild: {}", guild_id);
        Ok(guilds::table
            .filter(guilds::guild_id.eq(guild_id))
            //.filter(guilds::active.eq(true))
            //.filter(guilds::rules_message_id.is_not_null())
            //.filter(guilds::rules_channel_id.is_not_null())
            //.filter(guilds::member_role.is_not_null())
            .get_result::<Guild>(connection)?
            .into())
    }

    pub fn from_guild_id(connection: &PgConnection, guild_id: i64) -> Result<Self, BotError> {
        Ok(guilds::table
            .filter(guilds::guild_id.eq(guild_id))
            .get_result::<Self>(connection)?)
    }

    pub fn update<Update: Into<GuildUpdate>>(
        &self,
        connection: &PgConnection,
        change: Update,
    ) -> Result<Self, BotError> {
        Ok(match change.into() {
            GuildUpdate::RulesMessageUpdate(u) => {
                diesel::update(self).set(u).get_result(connection)
            }
            GuildUpdate::RulesContentUpdate(u) => {
                diesel::update(self).set(u).get_result(connection)
            }
            GuildUpdate::ModeratorRoleUpdate(u) => {
                diesel::update(self).set(u).get_result(connection)
            }
            GuildUpdate::MemberRoleUpdate(u) => diesel::update(self).set(u).get_result(connection),
            GuildUpdate::RulesChannelUpdate(u) => {
                diesel::update(self).set(u).get_result(connection)
            }
            GuildUpdate::LogsChannelUpdate(u) => diesel::update(self).set(u).get_result(connection),
            GuildUpdate::ClearModeratorRole => diesel::update(self)
                .set(guilds::admin_role.eq(Option::<i64>::None))
                .get_result(connection),
            GuildUpdate::UnbindMessage => diesel::update(self)
                .set((
                    guilds::rules_message_id.eq(Option::<i64>::None),
                    guilds::rules_channel_id.eq(Option::<i64>::None),
                ))
                .get_result(connection),
            GuildUpdate::EnableBot => diesel::update(self)
                .set(guilds::active.eq(true))
                .get_result(connection),
            GuildUpdate::DisableBot => diesel::update(self)
                .set(guilds::active.eq(false))
                .get_result(connection),
        }?)
    }
}

#[derive(Insertable)]
#[table_name = "guilds"]
pub struct NewGuild {
    guild_id: i64,
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
        write!(
            f,
            "({}, {})",
            self.guild_id,
            if self.active { "active" } else { "inactive" }
        )
    }
}
