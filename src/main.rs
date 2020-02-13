#[macro_use]
extern crate diesel;

use crate::db::DbKey;
use std::env;
use diesel::prelude::*;

use serenity::{
    model::{event::ResumedEvent, gateway::Ready, id::{ChannelId,MessageId}, channel::Reaction},
    prelude::{EventHandler, Context},
    client::Client,
    framework::standard::{StandardFramework, macros::{group}}
};
use log::{error, info};
use std::sync::Arc;

mod schema;
mod models;
mod commands;
mod checks;
mod db;
mod errors;

use models::guilds::Guild;

use commands::{
    SET_MODERATOR_GROUP_COMMAND,
    DEBUG_COMMAND,
    HOOK_MESSAGE_COMMAND,
};

#[group]
#[commands(set_moderator_group,debug,hook_message)]
struct General;

struct Handler;
impl EventHandler for Handler {
    fn ready(&self, ctx: Context, ready: Ready) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        info!("{} is connected!", ready.user.name);
        ready.guilds.iter().for_each(
            |guild_status| {
                let guild_id = guild_status.id();
                println!("guildId: {}", guild_id.as_u64());
                Guild::new(format!("{}",guild_id.as_u64())).insert(&connection)
            }
        )
    }

    fn resume(&self, _: Context, resume: ResumedEvent) {
        info!("Resumed; trace: {:?}", resume.trace);
    }

    fn  reaction_add(&self, ctx: Context, add_reaction: Reaction) {
        let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        let guild = {
            let guild_id = add_reaction.channel(&ctx).unwrap().guild().unwrap().read().id;
            Guild::active_from_guild_id(
                &connection, 
                &guild_id.as_u64().to_string()
            ).expect("reaction not from active guild")
        };
        
        
        if guild.rules_message_id == add_reaction.message_id.to_string() {
            println!("aaa")
        } else {
            println!("bbb")
        };
    }
    fn  reaction_remove(&self, _ctx: Context, _add_reaction: Reaction) {
        // let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
        // msg.guild_id
        // .and_then(|guild_id| {
        //     Guild::from_guild_id(&connection, &guild_id.as_u64().to_string()).ok()
        // })
    }
    
    fn message_delete(
        &self,
        _ctx: Context,
        channel_id: ChannelId,
        deleted_message_id: MessageId
    ) {
        info!("channel deleted: {} {}", channel_id, deleted_message_id);
    }
}

fn main() {
    println!("Hello, world");
    env_logger::init();

    let pool = db::establish_connection();
    let connection = pool.get().expect("couldn't retrieve connection from the pool");
    let results = schema::guilds::table.load::<Guild>(&connection)
        .expect("Error loading posts");

    for guild in results {
        info!("{}", guild);
    }
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler)
        .expect("Error creating client");
    
    client.with_framework(StandardFramework::new()
        .configure(|c| c.prefix("~")) // set the bot's prefix to "~"
        .group(&GENERAL_GROUP));
    client.data.write().insert::<db::DbKey>(Arc::new(pool));
    if let Err(why) = client.start() {
        error!("An error occurred while running the client: {:?}", why);
    }
}