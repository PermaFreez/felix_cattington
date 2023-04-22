/*use serenity::{
    prelude::*,
    async_trait,
    all::{Message, Reaction, Ready},
};*/

use poise::serenity_prelude::*;

use rusqlite::Connection;

pub struct ReactionsHandler;

fn update_reactions(message: Message) {

    let reactions = serde_json::to_string(&message.reactions).unwrap();

    let conn = Connection::open("database.db").unwrap();

    let query = "UPDATE memes SET Reactions = ?1 WHERE Id = ?2;";

    conn.execute(&query, (&reactions, message.id.to_string())).unwrap();
}

#[async_trait]
impl EventHandler for ReactionsHandler {
    
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