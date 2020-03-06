use crate::checks::ADMIN_CHECK;
use crate::db::DbKey;
use crate::models::guilds::{
    Guild, GuildUpdate, LogsChannelUpdate, ModeratorGroupUpdate, RulesChannelUpdate,
    RulesContentUpdate, RulesMessageUpdate,
};
use log::{error, info};
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

fn get_single_value<T: std::fmt::Debug>(v: &[T]) -> Result<&T, SingleValueError> {
    info!("{}: {:?}", v.len(), v);
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
                .expect("couldn't update moderator group for guild");
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
pub fn set_rules_channel(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    if guild_conf.rules_message_id != None {
        msg.reply(
            &ctx,
            "can't set the rules channel while hooked to a message, please unbind message first",
        )?;
        return Ok(());
    }
    match args.single::<ChannelId>() {
        Ok(channel_id) => {
            guild_conf.update(
                &connection,
                RulesChannelUpdate {
                    rules_channel_id: *channel_id.as_u64() as i64,
                },
            )?;
            msg.reply(&ctx, format!("rules channel is now {}", channel_id))?;
        }
        Err(err) => {
            info!("argument is not a mention: {}", err);
            msg.reply(&ctx, "first argument should be a channel mention")?;
        }
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
pub fn set_logs_channel(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    match args.single::<ChannelId>() {
        Ok(channel_id) => {
            guild_conf.update(
                &connection,
                LogsChannelUpdate {
                    log_channel_id: *channel_id.as_u64() as i64,
                },
            )?;
            msg.reply(&ctx, format!("logs channel is now {}", channel_id))?;
        }
        Err(err) => {
            info!("argument is not a mention: {}", err);
            msg.reply(&ctx, "first argument should be a channel mention")?;
        }
    };

    Ok(())
}

#[command]
#[only_in(guilds)]
pub fn clear_moderator_group(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();
    guild_conf.update(&connection, GuildUpdate::ClearModeratorGroup)?;
    msg.reply(&ctx, "cleared moderator group")?;
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
//#[display_in_help(false)]
pub fn status(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    let status_str = MessageBuilder::new()
        .push_bold("rules:\n")
        .push(&guild.rules)
        .push("\n")
        .push_bold("rules channel: ")
        .push(match guild.rules_channel_id {
            Some(channel_id) => channel_id.to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("rules message: ")
        .push(match guild.rules_message_id {
            Some(message_id) => message_id.to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("moderator role: ")
        .push(match guild.admin_role {
            Some(admin_role) => admin_role.to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("log channel: ")
        .push(match guild.log_channel_id {
            Some(log_channel_id) => log_channel_id.to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("reactions: ")
        .push(&guild.reaction_ok)
        .push(" / ")
        .push(&guild.reaction_reject)
        .push("\n")
        .push_bold("strict: ")
        .push(if guild.strict { "true" } else { "false" })
        .push("\n")
        .build();
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.title(format!(
                "guild rules - ({})",
                if guild.active { "active" } else { "inactive" }
            ));
            e.description(status_str);
            e.colour(if guild.active { 0x00_ff_00 } else { 0xff_00_00 });
            e
        });
        m
    })?;
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
        (Some(m_id), Some(c_id)) => {
            msg.reply(&ctx, "rules bot enabled")?;
            guild.update(&connection, GuildUpdate::EnableBot)?;
        }
        (None, Some(channel_id)) => {
            match ChannelId(*channel_id as u64).send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("welcome to this server, please read and accept these rules to proceed to the channels");
                    e.description(&guild.rules);
                    e
                });
                m
            }) {
                Ok(rules_message) => {
                    guild.update(
                        &connection, 
                    RulesMessageUpdate {
                        rules_message_id: *rules_message.id.as_u64() as i64,
                        rules_channel_id: *channel_id,
                        rules: guild.rules.clone(),
                    },
                        )?;
                    msg.reply(
                    &ctx,
                    format!("rules message created in channel {}", channel_id),
                )?;},
                Err(error) => {
                    error!("couldn't create rules message, {}", error);
                    msg.reply(&ctx, "an error occured")?;
                }
            };
        }
        (Some(m_id), None) => {
            msg.reply(
                &ctx,
                format!(
                    "setup invalid: unexpected message id {} without registered rules channel",
                    m_id
                ),
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
pub fn unbind_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    if guild.active {
        msg.reply(
            &ctx,
            "please disable the bot before unbinding to the rules message",
        )?;
    } else {
        msg.reply(&ctx, "bot config cleared")?;
        guild.update(&connection, GuildUpdate::UnbindMessage)?;
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
        guild.update(&connection, GuildUpdate::DisableBot)?;
        Ok(())
    }
}
