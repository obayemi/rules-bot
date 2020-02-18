use crate::checks::ADMIN_CHECK;
use crate::db::DbKey;
use crate::models::guilds::{
    Guild, GuildUpdate, ModeratorGroupUpdate, RulesContentUpdate, RulesMessageUpdate,
};
use log::info;
use serenity::framework::standard::CommandError;

use serenity::framework::standard::{
    //macros::{check, command, group, help},
    macros::command,
    Args,
    CommandResult,
};
use serenity::model::{
    channel::Message,
    id::{ChannelId, MessageId},
};
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

enum SingleValueError {
    NoValue,
    MultipleValue,
}

fn get_single_value<T>(v: &[T]) -> Result<&T, SingleValueError> {
    match v.len() {
        0 => Err(SingleValueError::NoValue),
        1 => Ok(&v[0]),
        _ => Err(SingleValueError::MultipleValue),
    }
}

fn message_found(message_id: u64, channel_id: &ChannelId) -> String {
    MessageBuilder::new()
        .push("found message ")
        .push(message_id)
        .push(" in channel ")
        .mention(channel_id)
        .build()
}

#[command]
#[only_in(guilds)]
pub fn hook_message(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    if let Ok(message_id) = args.single::<u64>() {
        msg.reply(&ctx, format!("searching message {}", message_id))
            .expect("failed to send message");
        let channels = guild.channels(&ctx).expect("guild should have channels");
        if let Some(message) = channels
            .iter()
            .find_map(|(_cid, c)| c.message(&ctx, message_id).ok())
        {
            guild_conf
                .update(
                    &connection,
                    RulesMessageUpdate {
                        rules_message_id: message_id as i64,
                        rules_channel_id: message.channel_id.into(),
                        rules: message.content,
                    },
                )
                .unwrap();
            let reply = message_found(message_id, &message.channel_id);
            msg.reply(&ctx, reply).expect("failed to send message");
            Ok(())
        } else {
            msg.reply(&ctx, format!("message with id {} not found", message_id))
                .expect("failed to send message");
            info!("message with id {} not found", message_id);
            Err(CommandError(format!(
                "message with id {} not found",
                message_id
            )))
        }
    } else {
        msg.reply(&ctx, "missing message id")
            .expect("faild to send message");
        info!("missing message id");
        Err(CommandError("missing message id".to_string()))
    }
}

#[command]
#[only_in(guilds)]
pub fn update_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    match (guild_conf.rules_channel_id, guild_conf.rules_message_id) {
        (Some(rules_channel_id), Some(rules_message_id)) => {
            let channel_id = ChannelId(rules_channel_id as u64);
            let message_id = MessageId(rules_message_id as u64);
            match channel_id.message(&ctx, message_id) {
                Ok(message) => {
                    channel_id
                        .edit_message(&ctx, message_id, |m| m.content(guild_conf.rules))
                        .unwrap();
                    msg.reply(&ctx, "updated")?;
                }
                Err(e) => {
                    msg.reply(&ctx, "msg not found")?;
                    info!("{:?}", e);
                }
            }
        }
        (_, _) => {
            msg.reply(ctx, "no rules message tracked")?;
        }
    };
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Admin)]
pub fn set_moderator_group(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    match get_single_value(&msg.mention_roles) {
        Ok(role_id) => {
            guild_conf
                .update(
                    &connection,
                    ModeratorGroupUpdate {
                        admin_role: *role_id.as_u64() as i64,
                    },
                )
                .expect(&format!(
                    "couldn't update moderator group for guild {}",
                    guild.id
                ));
            msg.reply(&ctx, "ok")?;
            info!(
                "moderator group for guild {} set to {}",
                guild.id,
                *role_id.as_u64() as i64
            );
        }
        Err(SingleValueError::NoValue) => {
            msg.reply(&ctx, "no role mentionned")?;
        }
        Err(SingleValueError::MultipleValue) => {
            msg.reply(&ctx, "too many roles mentioned")?;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub fn set_rules(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    //info!("{}", args.trimmed().message());
    let new_rules = args.trimmed().message().to_string();
    msg.reply(&ctx, format!("new rules {}", new_rules))?;
    guild.update(&connection, RulesContentUpdate { rules: new_rules })?;
    Ok(())
}

#[command]
#[only_in(guilds)]
//#[display_in_help(false)]
pub fn debug(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    msg.reply(&ctx, format!("```json\n{:?}\n```", guild))?;
    Ok(())
}

#[command]
#[only_in(guilds)]
pub fn enable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    if guild.active {
        msg.reply(&ctx, "rules already enabled")?;
        return Ok(());
    }
    match (&guild.rules_message_id, &guild.rules_channel_id) {
        (Some(_), _) => {
            msg.reply(&ctx, "rules bot enabled")?;
            guild.update(&connection, GuildUpdate::EnableBot)?;
        }
        (_, Some(channel_id)) => {
            msg.reply(
                &ctx,
                format!("rules message created in channel {}", channel_id),
            )?;
        }
        (None, None) => {
            msg.reply(&ctx, "setup incomplete")?;
        }
    }
    Ok(())
}

#[command]
#[only_in(guilds)]
pub fn disable(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    if !guild.active {
        msg.reply(&ctx, "rules not enabled")?;
        Ok(())
    } else {
        msg.reply(&ctx, "rules bot disabled")?;
        guild.update(&connection, GuildUpdate::DisableBot);
        Ok(())
    }
}
