use tracing::info;
use sqlx::postgres::{PgPoolOptions,PgPool};
use std::env;
use anyhow::anyhow;

pub struct Data {
    pool: PgPool
}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

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
            show_context_menu_commands: true,
            ..Default::default()
        },
    )
    .await?;
    Ok(())
}

/// replies text sent
#[poise::command(prefix_command, slash_command)]
pub async fn ping(
    ctx: Context<'_>,
    #[description = "name"] msg: String,
    ) -> Result<(), Error>{
    ctx.say(msg).await?;
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

    info!(test_value = 2, "aaa");
    poise::Framework::build()
        .token(std::env::var("DISCORD_TOKEN").unwrap())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data{pool}) }))
        .options(poise::FrameworkOptions {
            // configure framework here
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: std::env::var("DISCORD_PREFIX").ok(),
                ..Default::default()
            },
            commands : vec![
                ping(),
                help(),
                register(),
            ],
            ..Default::default()
        })
        .run().await.unwrap();
}
