use std::env;
use log::info;
use poise::CreateReply;
use rusqlite::Connection;

use crate::response::getembed;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context2<'a> = poise::Context<'a, Data, Error>;

/// Megmutat minden eddig létező formátumot és hozzá tartozó példát
#[poise::command(slash_command)]
pub async fn formatumok(ctx: Context2<'_>) -> Result<(), Error> {
    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).expect("Failed to open database!");

    let query = "SELECT Name, Example FROM templates";

    let mut description = String::new();
    {
        let mut stmt = conn.prepare(query).unwrap();
        for row in stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap() {
            let template: (String, String) = row.unwrap();
            if description.is_empty() {
                description = format!("`{}` {}", template.0, template.1);
            } else {
                description = format!("{}\n`{}` {}", description, template.0, template.1);   
            }
        }
    }
    let embed = getembed("Valami", description);

    let reply = CreateReply::new().embed(embed);

    ctx.send(reply).await.unwrap();

    info!("{} használta a /formatumok parancsot!", ctx.author().id);
    Ok(())
}