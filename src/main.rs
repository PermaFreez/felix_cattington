mod messages;
mod turnoff;
mod reactions;
mod search;
mod tag;
mod falspoz;
mod logger;
mod belep;
mod help;
mod quicktag;

use std::{env, collections::HashSet};
use dotenv::dotenv;
use log::info;

use poise::serenity_prelude::{GatewayIntents, UserId};

pub struct Data {}


#[tokio::main]
async fn main() {
    logger::setup_logger().unwrap();
    info!("##################");
    info!("Program elindítva!");
    info!("##################");

    dotenv().ok();
    create_db().await;

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find environment variable!");

    let mut owners: HashSet<UserId> = HashSet::new();
    owners.insert(UserId::new(418109786622787604));

    let framework = poise::FrameworkBuilder::default()
        .options(poise::FrameworkOptions {
            commands: vec![search::search_all(),
                search::search_random(),
                tag::tag(),
                help::help()],
            owners: owners,
            ..Default::default()
        })
        .token(&token)
        .client_settings(|client| { client
            .event_handler(messages::InformerHandler)
            .event_handler(falspoz::FalsPozHandler)
            .event_handler(reactions::ReactionsHandler)
            .event_handler(turnoff::TurnoffHandler)
            .event_handler(belep::BelepHandler)
            .event_handler(quicktag::QuickTagHandler)
        })
        .intents(GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGE_REACTIONS)
        .user_data_setup(|ctx, _ready, framework| {
            Box::pin(async move {
                let create_commands = poise::builtins::create_application_commands(&framework.options().commands);
                poise::serenity_prelude::Command::set_global_commands(ctx, create_commands).await.unwrap();
                Ok(Data {})
            })
        });

    framework.run().await.unwrap();
}

async fn create_db() {
    let creation_query1 = "CREATE TABLE IF NOT EXISTS memes(FileName varchar(255) PRIMARY KEY, Id varchar(255), 
        Link varchar(255), Tags varchar(65535), Reactions varchar(65535), Locked boolean);";
    let creation_query2 = "CREATE TABLE IF NOT EXISTS users(UserId varchar(255) PRIMARY KEY, Memes varchar(1023));";
    let creation_query3 = "CREATE TABLE IF NOT EXISTS tags(Tag varchar(255) PRIMARY KEY, Memes varchar(65535));";
    let creation_query4 = "CREATE TABLE IF NOT EXISTS turnoff(UserId varchar(255) PRIMARY KEY);";
    let creation_query5 = "CREATE TABLE IF NOT EXISTS banned(UserId varchar(255) PRIMARY KEY);";
    let creation_query6 = "CREATE TABLE IF NOT EXISTS quicktag(UserId varchar(255) PRIMARY KEY, FileName varchar(255));";

    let conn = rusqlite::Connection::open("database.db").unwrap();

    conn.execute(creation_query1, ()).unwrap();
    conn.execute(creation_query2, ()).unwrap();
    conn.execute(creation_query3, ()).unwrap();
    conn.execute(creation_query4, ()).unwrap();
    conn.execute(creation_query5, ()).unwrap();
    conn.execute(creation_query6, ()).unwrap();

    info!("Adatbázisok létrehozva (IF NOT EXISTS).");
}