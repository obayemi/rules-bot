#[macro_use]
extern crate diesel;

use crate::db::DbKey;
use std::env;

use log::{error, info, warn};
use serenity::client::{Client, Context, EventHandler};
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
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

use models::guilds::{ActiveGuild, Guild};

use checks::*;
use commands::*;

#[group]
#[only_in(guilds)]
#[checks(Moderator)]
#[commands(
    set_moderator_role,
    clear_moderator_role,
    debug,
    input_rules,
    hook_message,
    set_rules,
    enable,
    disable,
    update_message,
    unbind_message,
    status,
    set_rules_channel,
    set_logs_channel,
    set_member_role
)]
struct General;

use diesel::PgConnection;
fn get_active_guild(ctx: &Context, connection: &PgConnection, reaction: &Reaction) -> ActiveGuild {
    let channel = reaction.channel(ctx);
    let guild_channel_rwl = channel.unwrap().guild().unwrap();
    let guild_channel = guild_channel_rwl.read();
    let guild_id = guild_channel.guild_id;
    Guild::active_from_guild_id(&connection, *guild_id.as_u64() as i64)
        .expect("reaction not from active guild")
}

fn member_details(ctx: &Context, member: &Member) -> String {
    let user = member.user_id().to_user(ctx).unwrap();
    MessageBuilder::new()
        .push_bold("member: ")
        .mention(member)
        .push("\n")
        .push_bold("display_name: ")
        .push_line(member.display_name().as_ref())
        .push_bold("tag: ")
        .push_line(user.tag())
        .build()
}

fn log_event<C: Into<Colour>>(
    ctx: &Context,
    member: &Member,
    logs_channel_id: Option<i64>,
    event: &str,
    color: C,
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
            })
            .unwrap();
    }
}

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        info!("{} is connected!", ready.user.name);
        ready.guilds.iter().for_each(|guild_status| {
            let guild_id = guild_status.id();
            println!("guildId: kkk({})", guild_id.as_u64());
            Guild::new(guild_id.into()).insert(&connection)
        })
    }

    fn resume(&self, _: Context, resume: ResumedEvent) {
        info!("Resumed; trace: {:?}", resume.trace);
    }

    fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        let guild = get_active_guild(&ctx, &connection, &reaction);

        if guild.rules_message_id as u64 != *reaction.message_id.as_u64() {
            return;
        };
        let mut member = GuildId(guild.guild_id as u64)
            .member(&ctx, reaction.user_id)
            .unwrap();
        info!("reaction received");
        match reaction.emoji.as_data() {
            r if r == guild.reaction_ok => {
                info!("  => ok");
                member.add_role(&ctx, guild.member_role as u64).unwrap();
                log_event(
                    &ctx,
                    &member,
                    guild.log_channel_id,
                    "User accepted the rules",
                    0x00_ff_00,
                );
            }
            r if r == guild.reaction_reject => {
                info!("  => reject");
                member.kick(&ctx).unwrap();
                log_event(
                    &ctx,
                    &member,
                    guild.log_channel_id,
                    "User rejected the rules",
                    0xff_00_00,
                );
            }
            _ => {
                warn!("  => invalid reaction to rules message on");
            }
        }
    }
    fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        let guild = get_active_guild(&ctx, &connection, &reaction);

        if guild.rules_message_id as u64 != *reaction.message_id.as_u64() {
            return;
        };
        let mut member = GuildId(guild.guild_id as u64)
            .member(&ctx, reaction.user_id)
            .unwrap();

        if reaction.emoji.as_data() == guild.reaction_ok {
            member.remove_role(&ctx, guild.member_role as u64).unwrap();
            log_event(
                &ctx,
                &member,
                guild.log_channel_id,
                "User unaccepted the rules",
                0x00_00_ff,
            );
        }
    }

    fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId) {
        info!("message deleted: {} {}", channel_id, deleted_message_id);
        let _connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        //channel_id.send_message()
        //let guild = Guild::from_guild_id(&connection, deleted_message_id.guild_id.unwrap().into())
        //.expect("aaaa");
    }
}

#[help]
#[max_levenshtein_distance(3)]
fn my_help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, help_options, groups, owners)
}

fn main() {
    env_logger::init();

    let pool = db::establish_connection();
    //let connection = pool
    //.get()
    //.expect("couldn't retrieve connection from the pool");

    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");

    info!("discord client initialized");
    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix(&env::var("DISCORD_PREFIX").unwrap_or("~".to_string())))
            .group(&GENERAL_GROUP)
            .help(&MY_HELP),
    );
    client.data.write().insert::<db::DbKey>(Arc::new(pool));
    if let Err(why) = client.start() {
        error!("An error occurred while running the client: {:?}", why);
    }
}
