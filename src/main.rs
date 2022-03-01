use anyhow::anyhow;
use poise::serenity_prelude as serenity;
use poise::serenity_prelude::{ChannelId, Mentionable, MessageBuilder, MessageId, RoleId, Colour,Member};
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::str::FromStr;
use std::time::Duration;
use tracing::{info,warn};

#[derive(Debug)]
pub struct Data {
    pool: PgPool,
}

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


fn member_details(member: &Member) -> String {
    MessageBuilder::new()
        .push_bold("member: ")
        .mention(member)
        .push("\n")
        .push_bold("display_name: ")
        .push_line(member.display_name().as_ref())
        .push_bold("tag: ")
        .push_line(member.user.tag())
        .build()
}

async fn log_event(
    ctx: &serenity::Context,
    guild_id: serenity::GuildId,
    member: &Member,
    logs_channel_id: Option<i64>,
    event: &str,
    color: Colour,
) {
    info!(member=member.display_name().as_str(), guild_id=guild_id.0, event);
    if let Some(c_id) = logs_channel_id {
        ChannelId(c_id as u64)
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(event);
                    e.description(member_details(member));
                    e.colour(color);
                    e
                });
                m
            }).await
            .unwrap();
    }
}

async fn event_listener(
    ctx: &serenity::Context,
    event: &poise::Event<'_>,
    _framework: &poise::Framework<Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    match event {
        poise::Event::ReactionAdd { add_reaction } => {
            let guild_id = add_reaction.guild_id.ok_or(anyhow!("event has no guild_id"))?;
            let user_id = add_reaction.user_id.ok_or(anyhow!("event has no user_id"))?;
            let mut member = guild_id.member(&ctx, user_id).await?;

            let guild_config = sqlx::query!(
                "SELECT reaction_ok, reaction_reject, member_role_id, log_channel_id FROM guilds WHERE guild_id = $1 AND rules_message_id = $2 AND rules_channel_id = $3 AND member_role_id IS NOT NULL",
                guild_id.0 as i64,
                add_reaction.message_id.0 as i64,
                add_reaction.channel_id.0 as i64,
            )
            .fetch_optional(&data.pool)
            .await?
            .ok_or(anyhow!("not a tracked message"))?;

            info!(
                guild_id=guild_id.0,
                reaction=add_reaction.emoji.as_data().as_str(),
                member=format!("{:?}", add_reaction.member).as_str(),
                "reaction add"
            );
            match add_reaction.emoji.as_data() {
     
                r if r == guild_config.reaction_ok => {
                    info!("  => ok");
                    member.add_role(&ctx, guild_config.member_role_id.unwrap() as u64).await?;
                    log_event(
                        &ctx,
                        guild_id,
                        &member,
                        guild_config.log_channel_id,
                        "User accepted the rules",
                        Colour::from(0x00_ff_00),
                    ).await;
                }
                r if r == guild_config.reaction_reject => {
                    info!("  => reject");
                    member.kick(&ctx).await?;
                    add_reaction.delete(&ctx).await?;
                    add_reaction.channel_id.delete_reaction(&ctx, add_reaction.message_id, add_reaction.user_id, serenity::ReactionType::from_str(&guild_config.reaction_ok)?).await?;
                    log_event(
                        &ctx,
                        guild_id,
                        &member,
                        guild_config.log_channel_id,
                        "User rejected the rules",
                        Colour::from(0xff_00_00),
                    ).await;
                }
                _ => {
                    warn!("  => invalid reaction to rules message on");
                }
            };
        }
        poise::Event::ReactionRemove { removed_reaction } => {
            let guild_id = removed_reaction.guild_id.ok_or(anyhow!("event has no guild_id"))?;
            let user_id = removed_reaction.user_id.ok_or(anyhow!("event has no user_id"))?;
            let mut member = guild_id.member(&ctx, user_id).await?;

            let guild_config = sqlx::query!(
                "SELECT reaction_ok, member_role_id, log_channel_id FROM guilds WHERE guild_id = $1 AND rules_message_id = $2 AND rules_channel_id = $3 AND member_role_id IS NOT NULL",
                guild_id.0 as i64,
                removed_reaction.message_id.0 as i64,
                removed_reaction.channel_id.0 as i64,
            )
            .fetch_optional(&data.pool)
            .await?
            .ok_or(anyhow!("not a tracked message"))?;

            info!(
                guild_id=guild_id.0,
                reaction=removed_reaction.emoji.as_data().as_str(),
                member=format!("{:?}", removed_reaction.member).as_str(),
                "reaction remove"
            );
            match removed_reaction.emoji.as_data() {
                r if r == guild_config.reaction_ok => {
                    member.remove_role(&ctx, guild_config.member_role_id.unwrap() as u64).await?;
                    log_event(
                        &ctx,
                        guild_id,
                        &member,
                        guild_config.log_channel_id,
                        "User unaccepted the rules",
                        Colour::from(0x00_00_ff),
                    ).await;
                }
                _ => {
                    warn!("  => invalid reaction to rules message on");
                }
            };
        }
        poise::Event::GuildCreate { guild, is_new: _ } => {
            sqlx::query!(
                "INSERT INTO guilds (guild_id) VALUES ($1) ON CONFLICT DO NOTHING",
                guild.id.0 as i64
            )
            .execute(&data.pool)
            .await?;
            info!(
                guild_name = guild.name.as_str(),
                guild_id = guild.id.0 as u64,
                "guild created"
            )
        }
        poise::Event::Ready { data_about_bot } => {
            info!(
                bot_username = data_about_bot.user.name.as_str(),
                "bot connected!"
            )
        }
        _ => {}
    };
    Ok(())
}

fn permission_error_message(command_name: &str) -> String {
    format!("you do not have needed permissions to run command {} in this guild. If this is an error, please contact your guild administrator", command_name)
}

pub async fn on_error(err: poise::FrameworkError<'_, Data, Error>) {
    match err {
        poise::FrameworkError::CommandCheckFailed { error, ctx } => {
            info!(
                guild_id = ctx.guild_id().unwrap().0,
                error = format!("{:?}", error).as_str(),
                command = ctx.command().name,
                "comamnd check failed"
            );
            if let Some(_error) = error {
                ctx.send(|m| {
                    m.content(
                        "an unexpected error occured, please contact your guild administrator",
                    )
                })
                .await
                .ok();
            } else {
                ctx.send(|m| {
                    m.content(permission_error_message(&ctx.command().name))
                        .ephemeral(true)
                })
                .await
                .ok();
            }
        }
        poise::FrameworkError::Command { error, ctx: _ } => {
            info!(
                error = format!("{:?}", error).as_str(),
                "and error occured when running command"
            );
        }
        _ => {
            info!(
                error = format!("{:?}", err).as_str(),
                "unmanaged error happened"
            );
        }
    }
}

#[poise::command(prefix_command, hide_in_help)]
async fn register(ctx: Context<'_>, #[flag] global: bool) -> Result<(), Error> {
    poise::builtins::register_application_commands(ctx, global).await?;
    Ok(())
}

/// Show this help menu
#[poise::command(prefix_command, track_edits, slash_command)]
async fn help(
    ctx: Context<'_>,
    #[description = "Specific command to show help about"]
    #[autocomplete = "poise::builtins::autocomplete_command"]
    command: Option<String>,
) -> Result<(), Error> {
    poise::builtins::help(
        ctx,
        command.as_deref(),
        poise::builtins::HelpConfiguration {
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

pub async fn admin_check(ctx: Context<'_>) -> Result<bool, Error> {
    if ctx.framework().options().owners.contains(&ctx.author().id) {
        return Ok(true);
    }
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild is not registered in rules bot"))?;
    let member = guild_id.member(ctx.discord(), ctx.author().id).await?;
    Ok(member
        .permissions
        .map(|perms| perms.administrator())
        .unwrap_or(false))
}

pub async fn moderator_check(ctx: Context<'_>) -> Result<bool, Error> {
    // owners get a free pass for easier maintainance and help on configuration
    if ctx.framework().options().owners.contains(&ctx.author().id) {
        return Ok(true);
    }

    info!("running moderator check");
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("guild is not registered in rules bot"))?;

    let guild_name = guild_id.name(ctx.discord()).unwrap();
    info!(
        guild_id = guild_id.0,
        guild_name = guild_name.as_str(),
        "guildId found"
    );

    let member = guild_id.member(ctx.discord(), ctx.author().id).await?;
    if member.permissions.map(|perms| perms.administrator()) == Some(true) {
        return Ok(true);
    }

    let guild_config = match sqlx::query!(
        "SELECT admin_role_id FROM guilds WHERE guild_id = $1 AND admin_role_id IS NOT NULL",
        guild_id.0 as i64
    )
    .fetch_optional(&ctx.data().pool)
    .await? {
        Some(config) => config,
        _ => return Ok(false)
    };

    let admin_role_id = guild_config
        .admin_role_id
        .ok_or_else(|| anyhow!("sql no worky"))? as u64;

    info!(
        guild_id = guild_id.0,
        guild = guild_id
            .name(ctx.discord())
            .ok_or(anyhow!("failed getting guild name"))?
            .as_str(),
        moderators_role = admin_role_id,
        "moderators_check"
    );
    Ok(ctx
        .author()
        .has_role(ctx.discord(), guild_id, RoleId::from(admin_role_id))
        .await?)
}

async fn autocomplete_rule(
    ctx: Context<'_>,
    partial: String,
) -> impl Iterator<Item = poise::AutocompleteChoice<String>> {
    sqlx::query!(
        "SELECT name, rule FROM rules WHERE guild_id = $1 AND name LIKE $2",
        ctx.guild_id()
            .ok_or(anyhow!("guild only command"))
            .unwrap()
            .0 as i64,
        format!("%{}%", partial)
    )
    .fetch_all(&ctx.data().pool)
    .await
    .unwrap()
    .into_iter()
    .map(|rule| poise::AutocompleteChoice {
        name: rule.rule,
        value: rule.name,
    })
}

struct Rule {
    rule: String,
}

fn quoted_rules(rules: &[Rule]) -> String {
    rules
        .iter()
        .map(|r| format!("> - {}", r.rule))
        .collect::<Vec<String>>()
        .join("\n")
}

/// show bot configuration
#[poise::command(
    prefix_command,
    slash_command,
    category = "Management",
    check = "moderator_check"
)]
async fn status(ctx: Context<'_>) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;

    let guild_config = sqlx::query!(
        "SELECT rules_message_id, rules_channel_id, admin_role_id, log_channel_id, reaction_ok, reaction_reject, member_role_id FROM guilds WHERE guild_id = $1",
        guild_id.0 as i64
    ).fetch_one(&ctx.data().pool).await?;
    let rules = sqlx::query_as!(
        Rule,
        "SELECT rule FROM rules WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_all(&ctx.data().pool)
    .await?;
    info!(config = format!("{:?}", guild_config).as_str(), "status");
    let message = quoted_rules(&rules);

    let rules_channel_id = guild_config.rules_channel_id.map(|id| ChannelId(id as u64));

    let status_str = MessageBuilder::new()
        .push_bold_line("rules:")
        .push_line(message)
        .push("\n")
        .push_bold("rules channel: ")
        .push_line(match rules_channel_id {
            Some(channel_id) => channel_id.mention().to_string(),
            None => "None".to_string(),
        })
        .push_bold("rules message: ")
        .push(match (rules_channel_id, guild_config.rules_message_id) {
            (Some(channel_id), Some(message_id)) => {
                MessageId(message_id as u64).link(channel_id, Some(guild_id))
            }
            (_, Some(message_id)) => format!("Invalid message without channel: {}", message_id),
            (_, _) => "None".to_string(),
        })
        .push("\n")
        .push_bold("member role: ")
        .push(match guild_config.member_role_id {
            Some(member_role_id) => RoleId(member_role_id as u64).mention().to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("moderator role: ")
        .push(match guild_config.admin_role_id {
            Some(admin_role_id) => RoleId(admin_role_id as u64).mention().to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("log channel: ")
        .push(match guild_config.log_channel_id {
            Some(log_channel_id) => ChannelId(log_channel_id as u64).mention().to_string(),
            None => "None".to_string(),
        })
        .push("\n")
        .push_bold("reactions: ")
        .push(&guild_config.reaction_ok)
        .push(" / ")
        .push(&guild_config.reaction_reject)
        .push("\n")
        .build();
    ctx.send(|m| {
        m.embed(|e| {
            e.title("guild rules");
            e.description(status_str);
            e.colour(0x00_ff_00);
            e
        });
        m
    })
    .await?;
    Ok(())
}

/// Command to configure the rules-bot
#[poise::command(
    prefix_command,
    slash_command,
    aliases("config"),
    category = "Management",
    check = "moderator_check"
)]
async fn configure(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("This is the set  function!").await?;
    Ok(())
}

/// register a rule
#[poise::command(
    prefix_command,
    slash_command,
    check = "moderator_check",
    category = "Management"
)]
pub async fn set_rule(
    ctx: Context<'_>,
    #[description = "name"]
    #[autocomplete = "autocomplete_rule"]
    name: String,
    #[description = "rule"] rule: String,
    #[description = "extra context"] extra: Option<String>,
) -> Result<(), Error> {
    sqlx::query!(
            "INSERT INTO rules (guild_id, name, rule, extra) VALUES ($1, $2, $3, $4)
            ON CONFLICT (guild_id, name) DO UPDATE SET rule = EXCLUDED.rule, extra = EXCLUDED.extra RETURNING guild_id",
            ctx.guild_id().ok_or(anyhow!("guild only command"))?.0 as i64,
            name,
            rule,
            extra.unwrap_or_default(),
        )
        .fetch_optional(&ctx.data().pool)
        .await?
        .ok_or(anyhow!("rule registering failed"))?;

    ctx.say("rule registered").await?;
    Ok(())
}

/// drop a rule
#[poise::command(
    prefix_command,
    slash_command,
    check = "moderator_check",
    category = "Management"
)]
pub async fn drop_rule(
    ctx: Context<'_>,
    #[description = "name"]
    #[autocomplete = "autocomplete_rule"]
    name: String,
) -> Result<(), Error> {
    sqlx::query!(
        "DELETE FROM rules WHERE guild_id = $1 AND name = $2",
        ctx.guild_id().ok_or(anyhow!("guild only command"))?.0 as i64,
        name,
    )
    .execute(&ctx.data().pool)
    .await?;

    ctx.say("rule dropped").await?;
    Ok(())
}

/// Set channel where the bot will log its role management actions
#[poise::command(
    prefix_command,
    slash_command,
    rename = "set_logs_channel",
    category = "Management",
    check = "moderator_check"
)]
async fn set_logs_chanel(
    ctx: Context<'_>,
    #[description = "channel to log to"] channel: Option<serenity::Channel>,
) -> Result<(), Error> {
    // (if a message was previously tracked, it will remove the tracking,
    // run `track_message` again to start tracking an old message again)
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;
    info!(
        guild_id = guild_id.0,
        channel_id = channel.as_ref().map(|c| c.id().0),
        "setting up new logs channel"
    );
    sqlx::query!(
        "UPDATE guilds SET log_channel_id = $1 WHERE guild_id = $2",
        channel.as_ref().map(|c| c.id().0 as i64),
        guild_id.0 as i64,
    )
    .execute(&ctx.data().pool)
    .await?;
    if let Some(channel) = channel {
        ctx.say(format!("logs channel setup in {}", channel.mention()))
            .await?;
    } else {
        ctx.say("cleared logs channel").await?;
    }
    Ok(())
}

/// Manualy set a tracked rules message for the bot
#[poise::command(
    slash_command,
    context_menu_command = "track",
    rename = "track_message",
    category = "Management",
    check = "moderator_check"
)]
async fn set_tracked_rules_message(
    ctx: Context<'_>,
    #[description = "Message to track"] msg: serenity::Message,
) -> Result<(), Error> {
    // (if the message was not sent by the bot, make sure the bot user has edit permissions on the message to let it stay up to date)
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;
    info!(
        message_id = msg.id.0,
        guild_id = guild_id.0,
        channel_id = msg.channel_id.0,
        "tracking new rules message"
    );
    sqlx::query!(
        "UPDATE guilds SET (rules_message_id, rules_channel_id) = ($1, $2) WHERE guild_id = $3",
        msg.id.0 as i64,
        msg.channel_id.0 as i64,
        guild_id.0 as i64,
    )
    .execute(&ctx.data().pool)
    .await?;
    ctx.send(|m| {
        m.content(format!("now tracking message: {}", msg.link()))
            .embed(|e| {
                e.author(|a| a.icon_url(&msg.author.face()).name(&msg.author.name));
                e.description(msg.content);
                e
            })
    })
    .await?;
    Ok(())
}

/// Configure the role that will give adminitstration right on the bot for this guild
#[poise::command(
    prefix_command,
    slash_command,
    rename = "set_moderators",
    category = "Management",
    check = "admin_check"
)]
async fn set_moderators(
    ctx: Context<'_>,
    #[description = "moderators role"] role: Option<serenity::RoleId>, // can't use serenity::Role ?
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;
    info!("fooo: {:?}", role);
    info!(
        guild_id = guild_id.0,
        role_id = role.as_ref().map(|r| r.0),
        "setting up new admin role"
    );
    sqlx::query!(
        "UPDATE guilds SET admin_role_id = $1 WHERE guild_id = $2",
        role.as_ref().map(|r| r.0 as i64),
        guild_id.0 as i64,
    )
    .execute(&ctx.data().pool)
    .await?;
    if let Some(role) = role {
        ctx.say(format!("rules-bot admins are now {}", role.mention()))
            .await?;
    } else {
        ctx.say("cleared moderator_rold").await?;
    }
    Ok(())
}

/// Configure the role that the bot will give to members who accept the rules
#[poise::command(
    prefix_command,
    slash_command,
    rename = "set_member_role",
    category = "Management",
    check = "admin_check"
)]
async fn set_member_role(
    ctx: Context<'_>,
    #[description = "members role"] role: Option<serenity::RoleId>, // can't use serenity::Role ?
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;
    info!("fooo: {:?}", role);
    info!(
        guild_id = guild_id.0,
        role_id = role.as_ref().map(|r| r.0),
        "setting up new members role"
    );
    sqlx::query!(
        "UPDATE guilds SET member_role_id = $1 WHERE guild_id = $2",
        role.as_ref().map(|r| r.0 as i64),
        guild_id.0 as i64,
    )
    .execute(&ctx.data().pool)
    .await?;
    if let Some(role) = role {
        ctx.say(format!("rules-bot now give role {}", role.mention()))
            .await?;
    } else {
        ctx.say("cleared members role").await?;
    }
    Ok(())
}

#[poise::command(prefix_command, slash_command, aliases("rule"))]
async fn rules(
    ctx: Context<'_>,
    #[description = "rule to display"] 
    #[autocomplete = "autocomplete_rule"]
    rule: Option<String>,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;

    match rule {
        Some(rule_name) => {
            let rule = sqlx::query!(
                "SELECT name, rule, extra FROM rules WHERE guild_id = $1 AND name = $2",
                guild_id.0 as i64,
                rule_name
            )
            .fetch_optional(&ctx.data().pool)
            .await?;

            match rule {
                Some(rule) => {
                    ctx.send(|m| {
                        m.embed(|e| {
                            e.title(&rule.name).description(
                                MessageBuilder::new()
                                    .push_bold_line(&rule.rule)
                                    .push_line("")
                                    .push_line(&rule.extra)
                                    .build(),
                            )
                        })
                    })
                    .await?;
                }
                None => {
                    ctx.send(|m| m.content(format!("rule `{}` not found", rule_name)).ephemeral(true)).await?;
                }
            };
        }
        None => {
            let rules = sqlx::query!(
                "SELECT name, rule FROM rules WHERE guild_id = $1",
                guild_id.0 as i64
            )
            .fetch_all(&ctx.data().pool)
            .await?;
            ctx.send(|m| {
                m.embed(|e| {
                    e.title("rules").description(
                        rules
                            .iter()
                            .fold(&mut MessageBuilder::new(), |m, rule| {
                                m.push_bold_line(&rule.name)
                                    .push_line(&rule.rule)
                                    .push_line("")
                            })
                            .build(),
                    )
                })
            })
            .await?;
        }
    }
    Ok(())
}

#[poise::command(
    prefix_command,
    slash_command,
    category = "Management",
    check = "admin_check"
)]
async fn send_rule_message(
    ctx: Context<'_>,
    #[description = "channel"] channel: serenity::Channel,
) -> Result<(), Error> {
    let guild_id = ctx
        .guild_id()
        .ok_or_else(|| anyhow!("command can't be run outside a guild"))?;
    
    let guild_config = sqlx::query!(
            "SELECT reaction_ok, reaction_reject FROM guilds WHERE guild_id = $1",
            guild_id.0 as i64,
        )
        .fetch_optional(&ctx.data().pool)
        .await?
        .ok_or(anyhow!("guild not setup properly"))?;

    let rules = sqlx::query!(
        "SELECT rule FROM rules WHERE guild_id = $1",
        guild_id.0 as i64
    )
    .fetch_all(&ctx.data().pool)
    .await?;
    let message = channel.id().send_message(
        ctx.discord(), 
        |m| {
        m.embed(|e| {
            e.title("welcome to this server, please read and accept these  s to proceed to the channels").description(
                rules
                    .iter()
                    .fold(&mut MessageBuilder::new(), |m, rule| {
                        m
                            .push_line(format!("- {}", rule.rule));
                        m
                    })
                    .build(),
            )
        })
    }).await?;

    sqlx::query!(
        "UPDATE guilds SET (rules_message_id, rules_channel_id) = ($1, $2) WHERE guild_id = $3",
        message.id.0 as i64,
        message.channel_id.0 as i64,
        guild_id.0 as i64,
    )
    .execute(&ctx.data().pool)
    .await?;

    message.react(ctx.discord(), serenity::ReactionType::from_str(&guild_config.reaction_ok)?).await?;
    message.react(ctx.discord(), serenity::ReactionType::from_str(&guild_config.reaction_reject)?).await?;

    ctx.say(format!("rules message setup successfully in {}\n> {}", channel.mention(), message.link())).await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&env::var("DATABASE_URL").expect("missing `DATABASE_URL` env variable"))
        .await
        .expect("error connecting to the db");

    sqlx::migrate!().run(&pool).await.unwrap();

    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data { pool }) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            commands: vec![
                help(),
                register(),
                rules(),
                status(),
                set_tracked_rules_message(),
                set_moderators(),
                set_member_role(),
                set_logs_chanel(),
                set_rule(),
                drop_rule(),
                send_rule_message(),
            ],
            on_error: |error| Box::pin(on_error(error)),
            listener: |ctx, event, framework, user_data| {
                Box::pin(event_listener(ctx, event, framework, user_data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: std::env::var("DISCORD_PREFIX").ok(),
                edit_tracker: Some(poise::EditTracker::for_timespan(Duration::from_secs(3600))),
                ..Default::default()
            },
            ..Default::default()
        })
        .run()
        .await
        .unwrap();
}
