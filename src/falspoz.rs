use std::env;
use log::info;

use rusqlite::Connection;

use poise::serenity_prelude::{EventHandler,
    async_trait, Context, Interaction, CreateButton, ButtonStyle,
    CreateInteractionResponse, CreateInteractionResponseMessage
};

use crate::response::getembed;

pub struct FalsPozHandler;

#[async_trait]
impl EventHandler for FalsPozHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {
        
        let message_component = match intc.message_component() {
            Some(some) => some,
            None => return,
        };

        let split: Vec<&str> = message_component.data.custom_id.split(' ').collect();

        if split[0] == "fals-poz" {

            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();

            let query = "UPDATE memes SET Locked = ?1 WHERE FileName = ?2;";

            conn.execute(query, (false, &split[1])).unwrap();

            let repost_continue = format!("A repost-érzékelést fals-pozitívnak minősítetted. 
                Amennyiben fel szeretnéd venni az IT mém-könyvtárába, használd a `/tag `**`{}`**` <tagek>` parancsot!", &split[1]);

            let embed = getembed("Fals-pozitív jelentve", &repost_continue);
        
            let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).button(button));

            message_component.create_response(&ctx.http, reply).await.unwrap();

            info!("{} fals-pozitívként jelezte vissza a \"{}\" mémet!", &message_component.user.id, &split[1]);
        }
    }
}