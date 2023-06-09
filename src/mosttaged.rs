use std::env;
use log::info;

use poise::{CreateReply, serenity_prelude::{CreateButton, ButtonStyle, CreateActionRow}};

use rusqlite::Connection;

use crate::response::getembed;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Kiadja a 10 leghasználtabb cimkét
#[poise::command(slash_command, dm_only)]
pub async fn mosttaged(ctx: Context<'_>) -> Result<(), Error> {

    let tags = most_used_tags(10);

    let mut answer = String::new();
    for tag in tags {
        if answer.is_empty() {
            answer = format!("{} ({})", tag.0, tag.1);
        } else {
            answer = format!("{}, {} ({})", answer, tag.0, tag.1);
        }
    }

    let description = format!("A tíz leghasználtabb cimke a következő (ennyiszer): \n**{}**.", &answer);

    let embed = getembed("Top Cimkék", &description);

    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
    let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
    let reply = CreateReply::new().embed(embed).components(components);

    ctx.send(reply).await.unwrap();
    info!("{} használta a /mosttaged parancsot!", &ctx.author().id);

    Ok(())
}

/// Kiadja az összes eddig használt cimkét
#[poise::command(slash_command, dm_only)]
pub async fn alltagged(ctx: Context<'_>) -> Result<(), Error> {

    let tags = most_used_tags(0);

    let mut answers: Vec<String> = Vec::new(); {
        let mut answer = String::new();
        for tag in tags {
            if answer.len() > 1800 {
                answers.push(answer);
                answer = String::new();
            }
            if answer.is_empty() {
                answer = format!("{} ({})", tag.0, tag.1);
            } else {
                answer = format!("{}, {} ({})", answer, tag.0, tag.1);
            }
        }
        answers.push(answer);
    }

    let description = format!("Eddig ezeket a cimkéket használták (ennyiszer):");
    let mut cimkek: Vec<String> = Vec::new(); 
    for answer in answers {
        cimkek.push(format!("```\n{}\n```", &answer));
    }

    let embed = getembed("Cimkék", &description);

    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
    let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
    let reply = CreateReply::new().embed(embed).components(components);

    let mut tag_replys: Vec<CreateReply> = Vec::new();
    for cimke in cimkek {
        tag_replys.push(CreateReply::new().content(cimke));
    }

    ctx.send(reply).await.unwrap();
    for tag_reply in tag_replys {
        ctx.send(tag_reply).await.unwrap();
    }
    info!("{} használta a /alltagged parancsot!", &ctx.author().id);

    Ok(())
}

// quantity = 0 -> összes
fn most_used_tags(mut quantity: u32) -> Vec<(String, usize)> {
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

    if quantity == 0 {
        return tag_count;
    }

    if tag_count.len() < quantity as usize {
        quantity = tag_count.len() as u32;
    }

    let mut quantity_vec: Vec<(String, usize)> = Vec::new();

    for i in 0..quantity as usize - 1 {
        quantity_vec.push(tag_count[i].clone());
    }

    quantity_vec
}