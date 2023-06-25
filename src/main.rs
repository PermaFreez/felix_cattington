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
mod schedule;
mod user;
mod status;
mod mosttaged;
mod should_tag;
mod introduce;
mod tagop;
mod response;

use std::{env, collections::HashSet};
use dotenv::dotenv;
use log::info;
use rusqlite::Connection;

use poise::serenity_prelude::{GatewayIntents, UserId};

pub struct Data {}

const TAG_SEPARATOR: char = ',';
const UNLOCK_TIME: u64 = 0;

#[tokio::main]
async fn main() {
    logger::setup_logger().unwrap();
    info!("##################");
    info!("Program elindítva!");
    info!("##################");

    dotenv().ok();
    create_db().await;

    tag::unlock_all();

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find environment variable!");

    let mut owners: HashSet<UserId> = HashSet::new();
    owners.insert(UserId::new(418109786622787604));

    let framework = poise::FrameworkBuilder::default()
        .options(poise::FrameworkOptions {
            commands: vec![search::search_all(),
                search::search_random(),
                tag::tag(),
                help::help(),
                mosttaged::mosttaged(),
                mosttaged::alltagged(),
                should_tag::cimkezendo(),
                tagop::gettags(),
                quicktag::templates::formatumok()],
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
            .event_handler(quicktag::advanced::AdvancedHandler)
            .event_handler(quicktag::regtemp::RegTempHandler)
            .status(status::set_online_status())
            .activity(status::set_custom_status().unwrap())
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
    let queries: Vec<&str> = vec![
        "CREATE TABLE IF NOT EXISTS memes(FileName varchar(255) PRIMARY KEY, Id varchar(255), \
        Link varchar(255), Tags varchar(65535), Reactions varchar(65535), Locked boolean);",
        "CREATE TABLE IF NOT EXISTS users(UserId varchar(255) PRIMARY KEY, Memes varchar(1023));",
        "CREATE TABLE IF NOT EXISTS tags(Tag varchar(255) PRIMARY KEY, Memes varchar(65535));",
        "CREATE TABLE IF NOT EXISTS turnoff(UserId varchar(255) PRIMARY KEY);",
        "CREATE TABLE IF NOT EXISTS banned(UserId varchar(255) PRIMARY KEY);",
        "CREATE TABLE IF NOT EXISTS quicktag(UserId varchar(255) PRIMARY KEY, FileName varchar(255));",
        "CREATE TABLE IF NOT EXISTS upforgrabs(FileName varchar(255) PRIMARY KEY, AnnounceMessage varchar(255));",
        "CREATE TABLE IF NOT EXISTS introduced(UserId varchar(255) PRIMARY KEY);",
        "CREATE TABLE IF NOT EXISTS templates(Name varchar(255) PRIMARY KEY, Example varchar(255) NOT NULL);",
    ];

    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).unwrap();

    for query in queries {
        conn.execute(query, ()).unwrap();
    }

    info!("Adatbázisok létrehozva (IF NOT EXISTS).");
}