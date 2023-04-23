use std::env;
use dotenv::dotenv;
use log::info;

use rusqlite::Connection;

use poise::serenity_prelude::*;

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

            dotenv().ok();
            let conn = Connection::open("database.db").unwrap();

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");

            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let query = "UPDATE memes SET Locked = ?1 WHERE FileName = ?2;";

            conn.execute(query, (false, &split[1])).unwrap();

            let repost_continue = format!("A repost-érzékelést fals-pozitívnak minősítetted. 
                Amennyiben fel szeretnéd venni az IT mém-könyvtárába, használd a `/tag `**`{}`**` <tagek>` parancsot!", &split[1]);

            let embed = CreateEmbed::new().color(color)
             .title("Fals-pozitív jelentve")
             .description(repost_continue)
             .footer(CreateEmbedFooter::new(footer_text).icon_url(footer_icon));
        
            let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).button(button));

            message_component.create_response(&ctx.http, reply).await.unwrap();

            info!("{} fals-pozitívként jelezte vissza a \"{}\" mémet!", &message_component.user.id, &split[1]);
        }
    }
}