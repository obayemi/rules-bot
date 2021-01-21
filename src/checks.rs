use serenity::framework::standard::{macros::check, Args, CheckResult, CommandOptions};
use serenity::model::channel::Message;
use serenity::prelude::Context;
use tokio::task;

// This imports `typemap`'s `Key` as `TypeMapKey`.
use crate::db::DbKey;
use crate::models::guilds::Guild;

#[check]
#[name = "Admin"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn admin_check(
    ctx: &Context,
    msg: &Message,
    _args: &mut Args,
    _options: &CommandOptions,
) -> CheckResult {
    match msg.member(&ctx).await {
        Ok(member) => {
            member
                .permissions(&ctx)
                .await
                .map(|permissions| permissions.administrator())
                .map_or_else(
                    |_| CheckResult::Success,
                    |_| CheckResult::new_user("user is not administrator")
                )
        },
        Err(why) => {
            CheckResult::new_user(why)
        }
    }
}

#[check]
#[name = "Moderator"]
#[check_in_help(true)]
#[display_in_help(true)]
pub async fn moderator_check(
    ctx: &Context,
    msg: &Message,
    _: &mut Args,
    _: &CommandOptions,
) -> CheckResult {
    let pool = ctx.data.read().await.get::<DbKey>().unwrap().clone();
    let guild = msg.guild(&ctx).await.unwrap();

    let guild_conf = task::spawn_blocking(move || {
        let connection = pool.get().unwrap();
        Guild::from_guild_id(&connection, guild.id.into()).unwrap()
    }).await.unwrap();

    match msg.member(&ctx).await {
        Ok(member) => {
            let admin_permission= member
                .permissions(&ctx)
                .await
                .map(|permissions| permissions.administrator())
                .ok();

            if admin_permission == Some(true) {
                return CheckResult::Success;
            }
            if member.roles
                    .iter()
                    .any(|r| Some(*r.as_u64() as i64) == guild_conf.admin_role) {
                        CheckResult::Success
                    } else {
                        CheckResult::new_user("user don't have moderator role")
                    }
                },
        Err(why) => {
            CheckResult::new_user(why)
        }
    }
}
