use serenity::{
    framework::standard::{
        Args, CheckResult, CommandOptions,
        macros::{check},
    },
    model::channel::Message,
};

// This imports `typemap`'s `Key` as `TypeMapKey`.
use serenity::prelude::*;
use crate::models::guilds::Guild;
use crate::db::DbKey;

#[check]
#[name = "Admin"]
// Whether the check shall be tested in the help-system.
#[check_in_help(true)]
// Whether the check shall be displayed in the help-system.
#[display_in_help(true)]
pub fn admin_check(ctx: &mut Context, msg: &Message, _args: &mut Args, _options: &CommandOptions) -> CheckResult {
    msg.member(&ctx.cache).and_then(|member| {
        member.permissions(&ctx.cache).map(|permissions| 
            permissions.administrator()
        ).ok()
    })
    .unwrap_or(false).into()
}

#[check]
#[name = "Moderator"]
#[check_in_help(true)]
#[display_in_help(true)]
pub fn moderator_check(ctx: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> CheckResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    msg.member(&ctx.cache).and_then(|member| {
        let permissions = member.permissions(&ctx.cache)
            .map(|permissions| 
            permissions.administrator()
        ).ok();
        if permissions == None || permissions == Some(true) {
            return Some(true)
        }
        msg.guild_id
            .and_then(|guild_id| {
                Guild::from_guild_id(&connection, &guild_id.as_u64().to_string()).ok()
            })
            .and_then(|guild| {
                guild.admin_role
            })
            .map(|role_id| role_id.parse::<u64>().expect("admin role_id not a u64 value"))
            .map(|admin_role_id| {
                member.roles.iter().any(
                    |r| r.as_u64() == &admin_role_id
                )
            })
    })
    .unwrap_or(false).into()
}