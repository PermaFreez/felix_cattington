// Commands related to existing tags

use std::env;

use poise::CreateReply;

use log::info;
use rusqlite::Connection;

use crate::{Data, response::getembed};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Létező cimkék lekérdezése
#[poise::command(slash_command, dm_only)]
pub async fn gettags(ctx: Context<'_>,
    #[description = "A mém fájneve (feltöltési értesítőből)"] meme: String) -> Result<(), Error> {

        let db = env::var("DATABASE").unwrap();
        let conn = Connection::open(db).unwrap();

        let embed = getembed("Nem található", "Nem található mém a megadott fájlnévvel.");

        let mut reply = CreateReply::new().embed(embed);
        {
            let query = "SELECT Tags FROM memes WHERE FileName = ?1";
            let mut stmt = conn.prepare(query).unwrap();
            for row in stmt.query_map(&[("?1", &meme)], |row| Ok(row.get(0).unwrap())).unwrap() {
                let tags: String = row.unwrap();

                let embed = getembed("Tagek:",
                format!("**A megadott mém az alábbi cimkékkel rendelkezik:** ```{}```", tags));

                reply = CreateReply::new().embed(embed);
            }
        }

        info!("{} használta a /gettags parancsot!", &ctx.author().id);
        ctx.send(reply).await.unwrap();

        Ok(())
}