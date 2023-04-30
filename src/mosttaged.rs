use std::env;
use log::info;

use poise::{CreateReply, serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle, CreateActionRow}};

use rusqlite::Connection;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Kiadja a 10 leghasználtabb cimkét
#[poise::command(slash_command, dm_only)]
pub async fn mosttaged(ctx: Context<'_>) -> Result<(), Error> {

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    let tags = most_used_tags(10);

    let mut answer = String::new();
    for tag in tags {
        if answer.is_empty() {
            answer = format!("{} ({})", tag.0, tag.1);
        } else {
            answer = format!("{}, {} ({})", answer, tag.0, tag.1);
        }
    }

    let description = format!("A tíz leghasználtabb cimke a következő: \n**{}**.", &answer);

    let embed = CreateEmbed::new().color(color)
        .title("Top Tagek")
        .description(&description)
        .footer(CreateEmbedFooter::new(footer_text)
        .icon_url(footer_icon));

    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
    let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
    let reply = CreateReply::new().embed(embed).components(components);

    ctx.send(reply).await.unwrap();
    info!("{} használta a /mosttaged parancsot!", &ctx.author().id);

    Ok(())
}

fn most_used_tags(quantity: u32) -> Vec<(String, usize)> {
    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).unwrap();

    let mut tag_count: Vec<(String, usize)> = Vec::new();

    let query = "SELECT tag, memes FROM tags";
    let mut stmt = conn.prepare(&query).unwrap();
    for row in stmt.query_map([], |row| Ok((row.get(0).unwrap(), row.get(1).unwrap()))).unwrap() {
        let memes: (String, String) = row.unwrap();
        let memes_vec: Vec<&str> = serde_json::from_str(&memes.1).unwrap();
        tag_count.push((memes.0, memes_vec.len()));
    }

    tag_count.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", tag_count.len());

    let mut quantity_vec: Vec<(String, usize)> = Vec::new();

    for i in 0..quantity as usize - 1 {
        quantity_vec.push(tag_count[i].clone());
    }

    quantity_vec
}