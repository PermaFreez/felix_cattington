// Commands related to existing tags

use std::env;

use poise::{
    CreateReply,
    serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter}
};

use log::info;
use rusqlite::Connection;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Létező cimkék lekérdezése
#[poise::command(slash_command, dm_only)]
pub async fn gettags(ctx: Context<'_>,
    #[description = "A mém fájneve (feltöltési értesítőből)"] meme: String) -> Result<(), Error> {

        let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
        let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
    
        let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
            .expect("Color is to be defined in hex!"));

        let db = env::var("DATABASE").unwrap();
        let conn = Connection::open(db).unwrap();

        let embed = CreateEmbed::new()
        .title("Nem található")
        .description("Nem található mém a megadott fájlnévvel.")
        .footer(CreateEmbedFooter::new(&footer_text)
        .icon_url(&footer_icon))
        .color(color);

        let mut reply = CreateReply::new().embed(embed);
        {
            let query = "SELECT Tags FROM memes WHERE FileName = ?1";
            let mut stmt = conn.prepare(query).unwrap();
            for row in stmt.query_map(&[("?1", &meme)], |row| Ok(row.get(0).unwrap())).unwrap() {
                let tags: String = row.unwrap();

                let embed = CreateEmbed::new()
                .title("Tagek:")
                .description(format!("**A megadott mém az alábbi cimkékkel rendelkezik:** ```{}```", tags))
                .footer(CreateEmbedFooter::new(&footer_text)
                .icon_url(&footer_icon))
                .color(color);

                reply = CreateReply::new().embed(embed);
            }
        }

        info!("{} használta a /gettags parancsot!", &ctx.author().id);
        ctx.send(reply).await.unwrap();

        Ok(())
}