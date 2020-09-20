use crate::checks::ADMIN_CHECK;
use crate::db::DbKey;
use crate::diesel::{BelongingToDsl, RunQueryDsl};
use crate::models::{
    guilds::{
        Guild, GuildUpdate, LogsChannelUpdate, MemberRoleUpdate, ModeratorRoleUpdate,
        RulesChannelUpdate, RulesContentUpdate, RulesMessageUpdate,
    },
    rules::{NewRule, Rule},
};
use log::{error, info};
use regex::Regex;
use serde::Deserialize;
use serenity::framework::standard::CommandError;

use serenity::framework::standard::{
    //macros::{check, command, group, help},
    macros::command,
    Args,
    CommandResult,
};
use serenity::model::{
    channel::Message,
    id::{ChannelId, MessageId, RoleId},
};
use serenity::prelude::*;
use serenity::utils::MessageBuilder;

fn message_found(message_id: u64, channel_id: ChannelId) -> String {
    MessageBuilder::new()
        .push("found message ")
        .push(message_id)
        .push(" in channel ")
        .mention(&channel_id)
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
            let reply = message_found(message_id, message.channel_id);
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
                Ok(_message) => {
                    channel_id
                        .edit_message(&ctx, message_id, |m| {
                            m.content("");
                            m.embed(|e| {
                                e.title("welcome to this server, please read and accept these rules to proceed to the channels");
                                e.description(&guild_conf.rules);
                                e
                            });
                            m
                        })
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
pub fn set_moderator_role(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    match args.single::<RoleId>() {
        Ok(role_id) => {
            guild_conf.update(
                &connection,
                ModeratorRoleUpdate {
                    admin_role: *role_id.as_u64() as i64,
                },
            )?;

            msg.reply(
                &ctx,
                MessageBuilder::new()
                    .push("moderator role is now ")
                    .mention(&role_id)
                    .build(),
            )?;
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
pub fn set_member_role(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    match args.single::<RoleId>() {
        Ok(role_id) => {
            guild_conf.update(
                &connection,
                MemberRoleUpdate {
                    member_role: *role_id.as_u64() as i64,
                },
            )?;

            msg.reply(
                &ctx,
                MessageBuilder::new()
                    .push("member role is now ")
                    .mention(&role_id)
                    .build(),
            )?;
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
            msg.reply(
                &ctx,
                MessageBuilder::new()
                    .push("rules channel is now ")
                    .mention(&channel_id)
                    .build(),
            )?;
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
            msg.reply(
                &ctx,
                MessageBuilder::new()
                    .push("logs channel is now ")
                    .mention(&channel_id)
                    .build(),
            )?;
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
pub fn clear_moderator_role(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();
    guild_conf.update(&connection, GuildUpdate::ClearModeratorRole)?;
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
    let rules = Rule::belonging_to(&guild)
        .load::<Rule>(&connection)
        .expect("could not get rules");
    msg.reply(&ctx, format!("```json\n{:?}\n{:?}```", guild, rules))?;
    Ok(())
}

use lazy_static::lazy_static;
lazy_static! {
    //static ref RULES_YAML_RE: Regex = Regex::new(r"```(?P<aa>.*)```").unwrap();
    static ref RULES_YAML_RE: Regex = Regex::new(r"(?s)```(yaml|)\n(?P<rules_yaml>.+)\n```").unwrap();
}

#[derive(Deserialize, Debug)]
struct RuleInput {
    name: String,
    rule: String,
    extra: Option<String>,
}

#[derive(Deserialize, Debug)]
struct OverallRules {
    preface: String,
    postface: String,
    rules: Vec<RuleInput>,
}

#[command]
#[only_in(guilds)]
pub fn input_rules(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");

    let rules_str = args.trimmed().message().to_string();
    let tmp = RULES_YAML_RE.captures(&rules_str).unwrap();
    //let rules =
    match serde_yaml::from_str::<OverallRules>(&tmp["rules_yaml"]) {
        Ok(rules) => {
            //println!("{:?}", &rules);
            guild
                .update_rules(
                    &connection,
                    &rules
                        .rules
                        .into_iter()
                        .map(|r| NewRule {
                            guild_id: guild.id,
                            name: r.name,
                            rule: r.rule,
                            extra: r.extra.unwrap_or("".into()),
                        })
                        .collect(),
                )
                .unwrap();
            //msg.reply(&ctx, format!("{:?}", rules)).unwrap();
        }
        Err(e) => {
            msg.reply(&ctx, format!("{}", e)).unwrap();
        }
    }

    //println!("{:?}", &tmp[rules_yaml]);
    //msg.reply(&ctx, format!("{:?}, ", &rules,)).unwrap();
    //msg.reply(&ctx, "test").unwrap();
    Ok(())
}

fn quoted_rules(rules: &str) -> String {
    "> ".to_string() + &rules.replace('\n', "\n> ")
}

#[command]
#[only_in(guilds)]
pub fn rules(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    let status_str = MessageBuilder::new()
        .push_bold_line("rules:")
        .push_line(quoted_rules(&guild.get_rules_detail(&connection)))
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
pub fn rule(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");

    let rule_name_arg = args
        .single::<String>()
        .map_err(|_| CommandError("require rule name".into()))?;

    let status_str = MessageBuilder::new()
        .push_line(
            &guild
                .get_rule_detail(&connection, &rule_name_arg)
                .map_err(CommandError)?,
        )
        .build();
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.description(status_str);
            e
        });
        m
    })?;
    Ok(())
}

#[command]
#[only_in(guilds)]
//#[display_in_help(false)]
pub fn status(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    info!(
        "guild id: {}",
        msg.guild_id.map(|id| id.to_string()).unwrap()
    );
    let status_str = MessageBuilder::new()
        .push_bold_line("rules:")
        .push_line(quoted_rules(&guild.get_rules_message(&connection)))
        .push("\n")
        .push_bold("rules channel: ")
        .push_line(match guild.rules_channel_id {
            Some(channel_id) => ChannelId(channel_id as u64).mention(),
            None => "None".to_string(),
        })
        .push_bold("rules message: ")
        .push(match (guild.rules_channel_id, guild.rules_message_id) {
            (Some(channel_id), Some(message_id)) => format!(
                "https://discordapp.com/channels/{}/{}/{}",
                guild.guild_id, channel_id, message_id
            ),
            (_, Some(message_id)) => format!("Invalid message: {}", message_id),
            (_, _) => "None".to_string(),
        })
        .push("\n")
        .push_bold("member role: ")
        .push(match guild.member_role {
            Some(member_role) => RoleId(member_role as u64).mention(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("moderator role: ")
        .push(match guild.admin_role {
            Some(admin_role) => RoleId(admin_role as u64).mention(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("log channel: ")
        .push(match guild.log_channel_id {
            Some(log_channel_id) => ChannelId(log_channel_id as u64).mention(),
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
            let rules_message = ChannelId(*c_id as u64)
                .message(&ctx, *m_id as u64)
                .map_err(|_|
                CommandError("an error occured with the bot message".into())
                )?;
            guild.update(&connection, GuildUpdate::EnableBot)?;
            rules_message.react(&ctx, guild.reaction_ok)?;
            rules_message.react(&ctx, guild.reaction_reject)?;
            msg.reply(&ctx, "rules bot enabled")?;
            Ok(())
        }
        (None, Some(channel_id)) => {
            let rules_message = ChannelId(*channel_id as u64).send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title("welcome to this server, please read and accept these rules to proceed to the channels");
                    e.description(&guild.rules);
                    e
                });
                m
            }).map_err(
                    |e| {
                    error!("couldn't create rules message, {:?}", e);
                    CommandError("an unexpected error occured".into())
                })?;
            guild.update(
                &connection, 
                RulesMessageUpdate {
                    rules_message_id: *rules_message.id.as_u64() as i64,
                    rules_channel_id: *channel_id,
                    rules: guild.rules.clone(),
                },
            )?;
            guild.update(&connection, GuildUpdate::EnableBot)?;
            rules_message.react(&ctx, guild.reaction_ok)?;
            rules_message.react(&ctx, guild.reaction_reject)?;
            msg.reply(
                &ctx,
                MessageBuilder::new().push("rules message created in channel ").mention(&rules_message.channel_id).build()
            )?;
            Ok(())
        }
        (Some(m_id), None) => {
                Err(CommandError(format!(
                    "setup invalid: unexpected message id {} without registered rules channel",
                    m_id
                ),
            ))
        }
        (None, None) => {
            Err(CommandError("Setup incomplete, bot requires a channel to print the rules, or a message to track. See `~help` and `~status`".into()))
        }
    }
}

#[command]
#[only_in(guilds)]
pub fn unbind_message(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    if guild.active {
        Err(CommandError(
            "please disable the bot before unbinding to the rules message".into(),
        ))
    } else {
        msg.reply(&ctx, "bot config cleared")?;
        guild.update(&connection, GuildUpdate::UnbindMessage)?;
        Ok(())
    }
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
