use crate::checks::{MODERATOR_CHECK};
// use crate::checks::{ADMIN_CHECK, MODERATOR_CHECK};
use crate::db::DbKey;
use tokio::task;
use futures::stream::{self,StreamExt};
use std::sync::Arc;
use lazy_static::lazy_static;
use crate::models::{
    guilds::{
        Guild, GuildUpdate, //LogsChannelUpdate,
        MemberRoleUpdate,
        ModeratorRoleUpdate,
        // RulesChannelUpdate, RulesContentUpdate,
        RulesMessageUpdate,
    },
    rules::{NewRule, Rule},
};
// use log::{error, info};
use regex::Regex;
use serde::Deserialize;

use serenity::framework::standard::{
    //macros::{check, command, group, help},
    macros::command,
    Args,
    CommandResult,
    CommandError,
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
#[checks(Moderator)]
pub async fn hook_message(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();

    let guild = msg.guild(&ctx).await.unwrap();

    let guild_id = guild.id;
    let p1 = pool.clone();
    let guild_conf = task::spawn_blocking(move || {
        let connection = &p1.get().unwrap();
        Guild::from_guild_id(connection, guild_id.into()).unwrap()
    }).await?;

    let message_id = args.single::<u64>()
        .map_err(|_| "missing message id")?;

    msg.reply(&ctx, format!("searching message {}", message_id)).await?;
    let channels = guild.channels(&ctx).await?;
    let messages = stream::iter(channels)
        .filter_map(|(_cid, c)| async move {
            c.message(&ctx, message_id).await.ok()
        })
        .take(1)
        .collect::<Vec<Message>>()
        .await;
    let message = messages
        .first()
        .ok_or_else(|| format!("message with id {} not found", message_id))?;
    let m = message.clone();
    task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        guild_conf.update(
            &connection,
            RulesMessageUpdate {
                rules_message_id: message_id as i64,
                rules_channel_id: m.channel_id.into(),
                rules: m.content.clone(),
            },
        ).map_err(|_| "could not save found message")
    }).await??;
    msg.reply(&ctx, message_found(message_id, message.channel_id)).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn update_message(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = task::spawn_blocking(move || {
        let connection = p1.get().unwrap();
        Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;

    match (guild.rules_channel_id, guild.rules_message_id) {
        (Some(rules_channel_id), Some(rules_message_id)) => {
            let channel_id = ChannelId(rules_channel_id as u64);
            let message_id = MessageId(rules_message_id as u64);
            let _message = channel_id
                .message(&ctx, message_id).await
                .map_err(|_| "rules message not found")?;

            let rule_message = task::spawn_blocking(move || {
                let connection = pool.get().unwrap();
                guild.get_rules_message(&connection)
            }).await?;

            channel_id
                .edit_message(&ctx, message_id, |m| {
                    m.content("");
                    m.embed(|e| {
                        e.title("welcome to this server, please read and accept these  s to proceed to the channels");
                        e.description(&rule_message);
                        e
                    });
                    m
                }).await?;
            msg.reply(&ctx, "updated").await?;
            Ok(())
        }
        (_, _) => Ok(())// Err("no rules message tracked"),
    }
}

#[command]
#[only_in(guilds)]
pub async fn set_moderator_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = task::spawn_blocking(move || {
      let connection = p1.get().unwrap();
      Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;

    let role_id = args
        .single::<RoleId>()
        .map_err(|_| "first argument should be a channel mention".to_string())?;

		task::spawn_blocking(move || {
      let connection = pool.get().unwrap();
      guild.update(
          &connection,
          ModeratorRoleUpdate {
              admin_role: *role_id.as_u64() as i64,
          },
      ).map_err(|e| e.to_string())
		}).await??;

    msg.reply(
        &ctx,
        MessageBuilder::new()
            .push("moderator role is now ")
            .mention(&role_id)
            .build(),
    ).await?;

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn set_member_role(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = task::spawn_blocking(move || {
      let connection = p1.get().unwrap();
      Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;

    let role_id = args
        .single::<RoleId>()
        .map_err(|_| "first argument should be a channel mention".to_string())?;
		task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        guild.update(
          &connection,
          MemberRoleUpdate {
              member_role: *role_id.as_u64() as i64,
          },
        ).map_err(|e| e.to_string())
  	}).await??;

    msg.reply(
        &ctx,
        MessageBuilder::new()
            .push("member role is now ")
            .mention(&role_id)
            .build(),
    ).await?;

    Ok(())
}

/*
#[command]
#[only_in(guilds)]
#[checks(Moderator)]
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
    let channel_id = args
        .single::<ChannelId>()
        .map_err(|_| "first argument should be a channel mention".to_string())?;
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

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub fn set_logs_channel(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, guild.id.into()).unwrap();

    let channel_id = args
        .single::<ChannelId>()
        .map_err(|_| "first argument should be a channel mention".to_string())?;
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

    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
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
#[checks(Moderator)]
pub fn set_rules(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, msg.guild_id.unwrap().into()).expect("aaaa");
    //info!("{}", args.trimmed().message());
    let new_rules = args.trimmed().message().to_string();
    msg.reply(&ctx, format!("new rules {}", new_rules))?;
    guild.update(&connection, RulesContentUpdate { rules: new_rules })?;
    Ok(())
}

*/
#[command]
#[only_in(guilds)]
#[checks(Moderator)]
//#[display_in_help(false)]
pub async fn debug(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let guild = task::spawn_blocking(move || {
      let connection = pool.get().unwrap();
      Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;
    msg.reply(&ctx, format!("```json\n{:?}```", guild)).await?;
    Ok(())
}

lazy_static! {
    static ref RULES_YAML_RE: Regex = Regex::new(r"(?s)```(yaml|)\n(?P<rules_yaml>.+)\n```").unwrap();
}

#[derive(Deserialize, Debug)]
struct RuleInput {
    name: String,
    rule: String,
    extra: Option<String>,
 }
// 
 #[derive(Deserialize, Debug)]
struct OverallRules {
    //preface: String,
    //postface: String,
    rules: Vec<RuleInput>,
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn clear_rules(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    task::spawn_blocking(move || -> Result<(),CommandError> {
      let connection = pool.get().unwrap();
      let guild = Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))?;
      guild.clear_rules(&connection).map_err(|e| format!("{}", e))?;
      Ok(())
    }).await??;

    msg.reply(&ctx, "rules cleared").await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn input_rules(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = task::spawn_blocking(move || {
      let connection = p1.get().unwrap();
      Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;

    let rules_str = args.trimmed().message().to_string();
    let tmp = RULES_YAML_RE.captures(&rules_str).unwrap();
    let rules = serde_yaml::from_str::<OverallRules>(&tmp["rules_yaml"])
        .map_err(|e| format!("{}", e))?;
    task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        guild.update_rules(
            &connection,
            &rules
            .rules
            .into_iter()
            .map(|r| NewRule {
                guild_id: guild.id,
                name: r.name,
                rule: r.rule,
                extra: r.extra.unwrap_or_else(|| "".into()),
            })
            .collect::<Vec<_>>(),
            ).map_err(|e| format!("{}", e))
    }).await??;
    msg.reply(&ctx, "rules set").await?;
    Ok(())
}

fn quoted_rules(rules: &str) -> String {
    "> ".to_string() + &rules.replace('\n', "\n> ")
}

#[command]
#[only_in(guilds)]
#[aliases(rules)]
pub async fn rule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = Arc::new(ctx.data.read().await.get::<DbKey>().unwrap().clone());
    let guild_id = msg.guild_id.unwrap().into();
    let status_str = task::spawn_blocking(move || -> CommandResult<String> {
        let connection = pool.get().unwrap();
        let guild = Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))?;
        match args.single::<String>() {
            Ok(rule_name_arg) => {
                let rule = guild.get_rule_detail(&connection, &rule_name_arg)?;
                Ok(MessageBuilder::new()
                    .push_line(rule)
                    .build())
            },
            Err(_) => {
                Ok(MessageBuilder::new()
                    .push_bold_line("rules:")
                    .push_line(&guild.get_rules_detail(&connection))
                    .build())
            }
        }
    }).await??;

    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.description(status_str);
            e
        });
        m
    }).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn set_rule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = Arc::new(ctx.data.read().await.get::<DbKey>().unwrap().clone());
    let guild_id = msg.guild_id.unwrap().into();
    let rule_name = args.single::<String>().map_err(|_| "require rule name and description `set_rule NAME DESCRIPTION`".to_string())?;
    let rule_description = args.single_quoted::<String>().map_err(|_| "require rule name and description `set_rule NAME DESCRIPTION [EXTRA]`".to_string())?;
    let rule_extra = args.single_quoted::<String>().unwrap_or_else(|_| "".to_string());


    let new_rule = task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        let guild = Guild::from_guild_id(&connection, guild_id).unwrap();
        let new_rule = NewRule{
            guild_id: guild.id,
            name: rule_name, 
            rule: rule_description,
            extra: rule_extra
        };
        new_rule.upsert(&connection)
    }).await??;
    let status_str = new_rule.format_long();
    msg.channel_id.send_message(&ctx, |m| {
        m.embed(|e| {
            e.description(status_str);
            e
        });
        m
    }).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn drop_rule(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let pool = Arc::new(ctx.data.read().await.get::<DbKey>().unwrap().clone());
    let guild_id = msg.guild_id.unwrap().into();
    let rule_name = args.single::<String>().map_err(|_| "require rule name".to_string())?;

    task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        let guild = Guild::from_guild_id(&connection, guild_id).unwrap();
        Rule::drop_rule(&rule_name, guild.id, &connection).map_err(|_| format!("could not delete rule `{}`", rule_name))
    }).await??;
    msg.reply(&ctx, format!("rule dropped")).await?;
    Ok(())
}

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
//#[display_in_help(false)]
pub async fn status(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = Arc::new(ctx.data.read().await.get::<DbKey>().unwrap().clone());
    let guild_id = msg.guild_id.unwrap().into();

    let (guild,message) = task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        let guild = Guild::from_guild_id(&connection, guild_id).unwrap();
        let message = quoted_rules(&guild.get_rules_message(&connection));
        (guild, message)
    }).await?;

    let status_str = MessageBuilder::new()
        .push_bold_line("rules:")
        .push_line(message)
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
    }).await?;
    Ok(())
}

use serenity::model::channel::ReactionType;
#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn enable(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = Arc::new(task::spawn_blocking(move || {
        Guild::from_guild_id(&p1.get().unwrap(), guild_id).unwrap()
    }).await?);
    if guild.active {
        msg.reply(&ctx, "rules already enabled".to_string()).await?;
        return Ok(());
    }
    match (guild.clone().rules_message_id, guild.clone().rules_channel_id) {
        (Some(m_id), Some(c_id)) => {
            let rules_message = ChannelId(c_id as u64)
                .message(&ctx, m_id as u64)
                .await
                .map_err(|_| "an error occured with the bot message".to_string())?;

            let p2 = pool.clone();
            let g = guild.clone();
            task::spawn_blocking(move || {
                g.update(&p2.get().unwrap(), GuildUpdate::EnableBot)
            }).await?.map_err(|e| e.to_string())?;
            rules_message.react(&ctx, ReactionType::from(guild.reaction_ok.chars().next().unwrap())).await?;
            rules_message.react(&ctx, ReactionType::from(guild.reaction_reject.chars().next().unwrap())).await?;
            msg.reply(&ctx, "rules bot enabled").await?;
            Ok(())
        }
        (None, Some(channel_id)) => {
            let rules_message = ChannelId(channel_id as u64).send_message(&ctx, |m| {
                m.embed(|e| {
                    e.title(format!("welcome to this server, please read and accept these rules (with \"{}\") to proceed to the channels", guild.reaction_ok));
                    e.description(&guild.rules);
                    e
                });
                m
            }).await
            .map_err(
                    |e| {
                    format!("couldn't create rules message, {:?}", e)
                })?;
            let g = guild.clone();
            let p2 = pool.clone();
            let rmid = rules_message.id;
            let _a: Result<(),String> = task::spawn_blocking(move || {
                let connection = &p2.get().unwrap();
                g.update(
                    connection,
                    RulesMessageUpdate {
                        rules_message_id: *rmid.as_u64() as i64,
                        rules_channel_id: channel_id,
                        rules: g.rules.clone(),
                    }).map_err(|_| "couldn't set message id".to_string())?;
                g.update(connection, GuildUpdate::EnableBot).map_err(|_| "couldn't set the guild as enabled".to_string())?;
                Ok(())
            }).await?;
            rules_message.react(&ctx, ReactionType::from(guild.reaction_ok.chars().next().unwrap())).await?;
            rules_message.react(&ctx, ReactionType::from(guild.reaction_reject.chars().next().unwrap())).await?;
            msg.reply(
                &ctx,
                MessageBuilder::new().push("rules message created in channel ").mention(&rules_message.channel_id).build()
            ).await?;
            Ok(())
        }
        (Some(m_id), None) => {
                Err(format!(
                    "setup invalid: unexpected message id {} without registered rules channel",
                    m_id
                ).into(),
            )
        }
        (None, None) => {
            Err("Setup incomplete, bot requires a channel to print the rules, or a message to track. See `~help` and `~status`".into())
        }
    }
}

/*
#[command]
#[only_in(guilds)]
#[checks(Moderator)]
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
*/

#[command]
#[only_in(guilds)]
#[checks(Moderator)]
pub async fn disable(ctx: &Context, msg: &Message) -> CommandResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild_id = msg.guild_id.unwrap().into();

    let p1 = pool.clone();
    let guild = task::spawn_blocking(move || {
        let connection = p1.get().unwrap();
        Guild::from_guild_id(&connection, guild_id).map_err(|e| format!("{}", e))
    }).await??;

    if !guild.active {
        msg.reply(&ctx, "bot already inactive").await?;
        Ok(())
        //Err("rules are not enabled".to_string()).into()
    } else {
        task::spawn_blocking(move || {
            guild.update(&pool.get().unwrap(), GuildUpdate::DisableBot).map_err(|e| format!("{}", e))
        }).await??;
        msg.reply(&ctx, "bot disabled").await?;
        Ok(())
    }
}
