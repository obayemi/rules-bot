#[macro_use]
extern crate diesel;

use crate::db::DbKey;
use std::env;

use log::{error, info, warn};
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{group, help, hook};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandError, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::prelude::{
    ChannelId, GuildId, Member, Message, MessageId, Reaction, Ready, ResumedEvent, UserId,
};
use serenity::utils::{Colour, MessageBuilder};
use std::collections::HashSet;
use std::sync::Arc;

mod checks;
mod commands;
mod db;
mod errors;
mod models;
mod schema;

use models::guilds::{ActiveGuild, Guild, NewGuild};

use commands::*;

#[group]
#[only_in(guilds)]
#[commands(
    set_moderator_role,
    //clear_moderator_role,
    debug,
    clear_rules,
    input_rules,
    hook_message,
    //set_rules,
    enable,
    //disable,
    //update_message,
    //unbind_message,
    rule,
    set_rule,
    drop_rule,
    status,
    //set_rules_channel,
    //set_logs_channel,
    set_member_role
)]
struct General;

use tokio::task;
use db::PgPool;

async fn get_active_guild(ctx: &Context, pool: Arc<PgPool>, reaction: &Reaction) -> ActiveGuild {
    let channel = reaction.channel(ctx).await;
    let guild_channel_rwl = channel.unwrap().guild().unwrap();
    let guild_channel = guild_channel_rwl;
    let guild_id = guild_channel.guild_id;
    task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        Guild::active_from_guild_id(&connection, *guild_id.as_u64() as i64)
            .expect("reaction not from active guild")
    }).await.unwrap()
}

fn member_details(_ctx: &Context, member: &Member) -> String {
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
    ctx: &Context,
    member: &Member,
    logs_channel_id: Option<i64>,
    event: &str,
    color: Colour,
) {
    if let Some(c_id) = logs_channel_id {
        ChannelId(c_id as u64)
            .send_message(ctx, |m| {
                m.embed(|e| {
                    e.title(event);
                    e.description(member_details(&ctx, member));
                    e.colour(color);
                    e
                });
                m
            }).await
            .unwrap();
    }
}

struct Handler;

#[serenity::async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, ready: Ready) {
        let connection = ctx.data.read().await.get::<DbKey>().unwrap().get().unwrap();
        info!("{} is connected!", ready.user.name);
        info!("list of joined guilds");
        ready.guilds.iter().for_each(|guild_status| {
            let guild_id = guild_status.id();
            info!("- {}", guild_id.as_u64());
            NewGuild::new(guild_id.into()).insert(&connection)
        })
    }

    async fn resume(&self, _: Context, resume: ResumedEvent) {
        info!("Resumed; trace: {:?}", resume.trace);
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let ctx_data = ctx.data.read().await;
        let pool = ctx_data.get::<DbKey>().unwrap();
        let guild = get_active_guild(&ctx, pool.clone(), &reaction).await;

        if guild.rules_message_id as u64 != *reaction.message_id.as_u64() {
            return;
        };
        let mut member = GuildId(guild.guild_id as u64)
            .member(&ctx, reaction.user_id.unwrap())
            .await
            .unwrap();
        info!("reaction received");
        match reaction.emoji.as_data() {
            r if r == guild.reaction_ok => {
                info!("  => ok");
                member.add_role(&ctx, guild.member_role as u64).await.unwrap();
                log_event(
                    &ctx,
                    &member,
                    guild.log_channel_id,
                    "User accepted the rules",
                    Colour::from(0x00_ff_00),
                ).await;
            }
            r if r == guild.reaction_reject => {
                info!("  => reject");
                member.kick(&ctx).await.unwrap();
                log_event(
                    &ctx,
                    &member,
                    guild.log_channel_id,
                    "User rejected the rules",
                    Colour::from(0xff_00_00),
                ).await;
            }
            _ => {
                warn!("  => invalid reaction to rules message on");
            }
        }
    }
    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let ctx_data = ctx.data.read().await;
        let pool = ctx_data.get::<DbKey>().unwrap();
        let guild = get_active_guild(&ctx, pool.clone(), &reaction).await;

        if guild.rules_message_id as u64 != *reaction.message_id.as_u64() {
            return;
        };
        let mut member = GuildId(guild.guild_id as u64)
            .member(&ctx, reaction.user_id.unwrap())
            .await
            .unwrap();

        if reaction.emoji.as_data() == guild.reaction_ok {
            member.remove_role(&ctx, guild.member_role as u64).await.unwrap();
            log_event(
                &ctx,
                &member,
                guild.log_channel_id,
                "User unaccepted the rules",
                Colour::from(0x00_00_ff),
            ).await;
        }
    }

    async fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId) {
        info!("message deleted: {} {}", channel_id, deleted_message_id);
        let _connection = ctx.data.read().await.get::<DbKey>().unwrap().get().unwrap();
        //channel_id.send_message()
        //let guild = Guild::from_guild_id(&connection, deleted_message_id.guild_id.unwrap().into())
        //.expect("aaaa");
    }
}

#[help]
#[max_levenshtein_distance(3)]
async fn my_help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    Ok(())
}

#[hook]
async fn after_hook(ctx: &Context, msg:&Message, cmd_name:&str, error: Result<(),CommandError>) {
    if let Err(why) = error {
        warn!("Error in {}: {:?}", cmd_name, why);
        msg.reply(&ctx, format!("{:?}", why)).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let pool = db::establish_connection();
    //let connection = pool
    //.get()
    //.expect("couldn't retrieve connection from the pool");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"))
        .event_handler(Handler)
        .framework(
            StandardFramework::new()
                .configure(|c| {
                    c.prefix(&env::var("DISCORD_PREFIX").unwrap_or_else(|_| "~".to_string()))
                })
                .after(after_hook)
                .group(&GENERAL_GROUP)
                .help(&MY_HELP),
            )
        .await.expect("Error creating client");
    info!("discord client initialized");
    {
        let mut data = client.data.write().await;
        data.insert::<db::DbKey>(Arc::new(pool));
    }
    if let Err(why) = client.start().await {
        error!("An error occurred while running the client: {:?}", why);
    }
}
