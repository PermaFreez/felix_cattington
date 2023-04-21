mod messages;

use std::env;
use dotenv::dotenv;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, prelude::Reaction},
    prelude::*,
};

const HELP_MESSAGE: &str = "help message";

const HELP_COMMAND: &str = "!help";

pub struct Handler;

fn update_reactions(message: Message) {
    let mut reactions: String = String::new();

    for reaction in &message.reactions {
        reactions = reactions + reaction.count.to_string().as_str() + reaction.reaction_type.to_string().as_str() + ";";
        
    }
    
    println!("{}", &reactions)

    //&message.link();
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if msg.content == HELP_COMMAND {
            if let Err(why) = msg.channel_id.say(&ctx.http, HELP_MESSAGE).await {
                println!("Error sending message: {:?}", why);
            }
        }
    }

    async fn reaction_add(&self, ctx: Context, reaction: Reaction) {
        let message = reaction.message(ctx.http).await.unwrap();

        update_reactions(message);
    }

    async fn reaction_remove(&self, ctx: Context, reaction: Reaction) {
        let message = reaction.message(ctx.http).await.unwrap();

        update_reactions(message);
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    create_db().await;

    let token = env::var("DISCORD_TOKEN").expect("Couldn't find environment variable!");

    let mut client = Client::builder(token, GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT | GatewayIntents::GUILD_MESSAGE_REACTIONS )
        .event_handler(messages::InformerHandler)
        .event_handler(messages::TaggingHandler)
        .event_handler(Handler)
        .await
        .expect("Err creating client");

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}

async fn create_db() {
    let creation_query1 = "CREATE TABLE IF NOT EXISTS memes(FileName varchar(255), Link varchar(255), Tags varchar(65535));";
    let creation_query2 = "CREATE TABLE IF NOT EXISTS users(UserId varchar(255), Memes varchar(1023));";
    let creation_query3 = "CREATE TABLE IF NOT EXISTS tags(Tag varchar(255), Memes varchar(65535));";

    let conn = rusqlite::Connection::open("database.db").unwrap();
    conn.execute(creation_query1, ()).unwrap();
    conn.execute(creation_query2, ()).unwrap();
    conn.execute(creation_query3, ()).unwrap();
}