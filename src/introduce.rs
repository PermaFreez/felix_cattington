use std::env;
use log::info;
use poise::serenity_prelude::{Context, CreateEmbed, CreateButton, ButtonStyle, CreateMessage, Message};
use rusqlite::Connection;

pub async fn introduce(msg: &Message, ctx: &Context) {
    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).unwrap();

    let mut count: u32 = 0;
    {
        let query = "SELECT Count(*) FROM introduced WHERE UserId = ?1";
        let mut stmt = conn.prepare(&query).unwrap();
        for row in stmt.query_map(&[("?1", &msg.author.id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
            count = row.unwrap();
        }
    }
    if count == 0 {
        let query2 = "INSERT INTO introduced (UserId) VALUES (?1)";
        conn.execute(&query2, &[("?1", &msg.author.id.to_string())]).unwrap();
        let query3 = "INSERT INTO turnoff (UserId) VALUES (?1)";
        conn.execute(&query3, &[("?1", &msg.author.id.to_string())]).unwrap();

        let description = "**Kérlek nézd meg ezt a rövid videót, hogy könnyedén megértsed, miért írt neked ez a bot!**";

        let embed: CreateEmbed = crate::response::getembed("Bemutatkozó", description);

        let button = CreateButton::new("visszairatkozas").label("Feliratkozás").style(ButtonStyle::Success);

        msg.author.dm(&ctx.http, CreateMessage::new()
        .content(env::var("INTRODUCE_LINK").unwrap())).await.unwrap();
        msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button)).await.unwrap();
        info!("A bot bemutatkozott {}-nak", &msg.author.id);
        return;
    } else {
        return;
    }
}