use crate::checks::{MODERATOR_CHECK, ADMIN_CHECK};
use crate::models::guilds::{Guild,ModeratorGroupUpdate,RulesMessageUpdate,RulesContentUpdate};
use serenity::model::{id::ChannelId, channel::Message};
use serenity::prelude::{Context};
use serenity::framework::standard::{
    CommandResult,
    Args,
    macros::{
        command,
    }
};
use serenity::utils::MessageBuilder;
use serenity::framework::standard::{CommandError};
use log::{info};
use crate::db::DbKey;

enum SingleValueError {
    NoValue,
    MultipleValue
}

fn get_single_value<T>(v: &Vec<T>) -> Result<&T, SingleValueError> {
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
#[checks(Moderator)]
pub fn hook_message(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, &guild.id.as_u64().to_string()).unwrap();

    if let Ok(message_id) = args.single::<u64>(){
        let channels = guild.channels(&ctx).expect("guild should have channels");
        if let Some(message) = channels.iter().find_map(|(_cid, c)| {
            c.message(&ctx, message_id).ok()
        }) {
            guild_conf.update(&connection, RulesMessageUpdate{
                rules_message_id: message_id.to_string(),
                rules_channel_id: message.channel_id.as_u64().to_string()
            }).unwrap();
            let reply = message_found(message_id, &message.channel_id);
            msg.reply(&ctx, reply).expect("failed to send message");
            Ok(())                
        } else {
            msg.reply(&ctx, format!("message with id {} not found", message_id)).expect("failed to send message");
            info!("message with id {} not found", message_id);
            Err(CommandError(format!("message with id {} not found", message_id)))
        }
    } else {
        msg.reply(&ctx, "missing message id").expect("faild to send message");
        info!("missing message id");
        Err(CommandError("missing message id".to_string()))
    }
}


#[command]
#[checks(Admin)]
pub fn set_moderator_group(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();

    let guild_lock = msg.guild(&ctx).unwrap();
    let guild = guild_lock.read();
    let guild_conf = Guild::from_guild_id(&connection, &guild.id.as_u64().to_string()).unwrap();

    match get_single_value(&msg.mention_roles) {
        Ok(role) => {
            guild_conf
                .update(
                    &connection, ModeratorGroupUpdate{admin_role:role.as_u64().to_string()}
                )
                .expect(&format!("couldn't update moderator group for guild {}", guild.id));
                msg.reply(&ctx, "ok")?;
                info!("moderator group for guild {} set to {}", guild.id, role.as_u64());
        },
        Err(SingleValueError::NoValue) => {msg.reply(&ctx, "no role mentionned")?;},
        Err(SingleValueError::MultipleValue) => {msg.reply(&ctx, "too many roles mentioned")?;},
    }
    Ok(())
}


#[command]
#[checks(Moderator)]
pub fn debug(ctx: &mut Context, msg: &Message) -> CommandResult {
    let connection = ctx.data.read().get::<DbKey>().unwrap().get().unwrap();
    let guild = Guild::from_guild_id(&connection, &msg.guild_id.unwrap().as_u64().to_string()).expect("aaaa");
    msg.reply(&ctx, format!("```json\n{:?}\n```", guild))?;
    Ok(())
}