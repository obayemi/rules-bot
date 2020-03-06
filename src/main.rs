#[macro_use]
extern crate diesel;

use crate::db::DbKey;
use diesel::prelude::*;
use std::env;

use log::{error, info};
use serenity::framework::standard::macros::{group, help};
use serenity::framework::standard::{
    help_commands, Args, CommandGroup, CommandResult, HelpOptions, StandardFramework,
};
use serenity::model::prelude::{Message, UserId};
use serenity::{
    client::{Client, Context, EventHandler},
    model::{
        channel::Reaction,
        event::ResumedEvent,
        gateway::Ready,
        id::{ChannelId, MessageId},
    },
};
use std::collections::HashSet;
use std::sync::Arc;

mod checks;
mod commands;
mod db;
mod errors;
mod models;
mod schema;

use models::guilds::Guild;

use checks::*;
use commands::*;

#[group]
#[only_in(guilds)]
#[checks(Moderator)]
#[commands(
    set_moderator_group,
    clear_moderator_group,
    debug,
    hook_message,
    set_rules,
    enable,
    disable,
    update_message,
    unbind_message,
    status,
    set_rules_channel,
    set_logs_channel
)]
struct General;

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

    fn reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        let guild = {
            let guild_id = add_reaction
                .channel(&ctx)
                .unwrap()
                .guild()
                .unwrap()
                .read()
                .id;
            Guild::active_from_guild_id(&connection, guild_id.into())
                .expect("reaction not from active guild")
        };

        if guild.rules_message_id == i64::from(add_reaction.message_id) {
            println!("aaa")
        } else {
            println!("bbb")
        };
    }
    fn reaction_remove(&self, _ctx: Context, _add_reaction: Reaction) {
        // let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        // msg.guild_id
        // .and_then(|guild_id| {
        //     Guild::from_guild_id(&connection, &guild_id.as_u64().to_string()).ok()
        // })
    }

    fn message_delete(&self, ctx: Context, channel_id: ChannelId, deleted_message_id: MessageId) {
        info!("message deleted: {} {}", channel_id, deleted_message_id);
        let _connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
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

    client.with_framework(
        StandardFramework::new()
            .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
            .group(&GENERAL_GROUP)
            .help(&MY_HELP),
    );
    client.data.write().insert::<db::DbKey>(Arc::new(pool));
    if let Err(why) = client.start() {
        error!("An error occurred while running the client: {:?}", why);
    }
}
