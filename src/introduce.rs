use std::env;
use log::info;
use poise::serenity_prelude::{Context, CreateEmbed, CreateEmbedFooter, Color, CreateButton, ButtonStyle, CreateMessage, Message};
use rusqlite::Connection;

pub async fn introduce(msg: &Message, ctx: &Context) {
    let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

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

        let embed: CreateEmbed = CreateEmbed::new().color(color)
            .title("Bemutatkozó")
            .description(description)
            .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));

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