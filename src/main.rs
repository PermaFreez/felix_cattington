mod messages;
mod turnoff;
mod reactions;

use std::env;
use dotenv::dotenv;

use serenity::{
    prelude::*,
};

#[tokio::main]
async fn main() {
    dotenv().ok();
    create_db().await;

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find environment variable!");

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGE_REACTIONS )
        .event_handler(messages::InformerHandler)
        .event_handler(messages::TaggingHandler)
        .event_handler(turnoff::TurnoffHandler)
        .event_handler(reactions::ReactionsHandler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn create_db() {
    let creation_query1 = "CREATE TABLE IF NOT EXISTS memes(FileName varchar(255), Id varchar(255) PRIMARY KEY, Tags varchar(65535), Reactions varchar(65535));";
    let creation_query2 = "CREATE TABLE IF NOT EXISTS users(UserId varchar(255) PRIMARY KEY, Memes varchar(1023));";
    let creation_query3 = "CREATE TABLE IF NOT EXISTS tags(Tag varchar(255) PRIMARY KEY, Memes varchar(65535));";
    let creation_query4 = "CREATE TABLE IF NOT EXISTS turnoff(UserId varchar(255) PRIMARY KEY);";

    let conn = rusqlite::Connection::open("database.db").unwrap();

    conn.execute(creation_query1, ()).unwrap();
    conn.execute(creation_query2, ()).unwrap();
    conn.execute(creation_query3, ()).unwrap();
    conn.execute(creation_query4, ()).unwrap();
}