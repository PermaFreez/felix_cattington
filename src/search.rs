use std::env;

use poise::{CreateReply, serenity_prelude::{CreateButton, ButtonStyle, CreateActionRow}};
use log::info;

use rand::prelude::*;
use rusqlite::Connection;

use crate::{Data, TAG_SEPARATOR, response::getembed};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Kiad egy random mémet az adott tag(ekk)el
#[poise::command(slash_command, dm_only)]
pub async fn search_all(ctx: Context<'_>,
    #[description = "Kereső tagek"] tagek: String) -> Result<(), Error> {

        let db = env::var("DATABASE").unwrap();
        let conn = Connection::open(db).unwrap();

        let ban_query = "SELECT Count(*) FROM banned WHERE UserId = ?1;";
        let mut is_banned = false;
        {
            let mut ban_stmt = conn.prepare(ban_query).unwrap();

            for row in ban_stmt.query_map(&[("?1", &ctx.author().id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
                let row: u32 = row.unwrap();
                if row == 1 {
                    is_banned = true;
                }
            }
        }

        if is_banned {
            let embed = getembed("Kitiltás",
            "Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.");
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} tiltva van, de megpróbált írni a botnak!", ctx.author().id);

            return Ok(());
        }

        let meme_links = search(&tagek.to_lowercase(), false);


        let description = format!("Itt vannak a leírásnak megfelelő mémek: {}", meme_links);
        let mut embed = getembed("Mém megtalálva:", description);
        if meme_links.is_empty() {
            embed = getembed("Nincs ilyen mém.", "A rendszer nem talált a kritériumoknak megfelelő mémet.");
        }
        
        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);

        let reply = CreateReply::new().embed(embed).components(vec![CreateActionRow::Buttons(vec![button])]);

        ctx.send(reply).await.unwrap();
        
        info!("{} rákeresett az összes ilyen mémre: {}", ctx.author().name, &tagek);

        Ok(())
}

/// Kiad egy random mémet az adott tag(ekk)el
#[poise::command(slash_command)]
pub async fn search_random(ctx: Context<'_>,
    #[description = "Kereső tagek"] tagek: String) -> Result<(), Error> {

        let db = env::var("DATABASE").unwrap();
        let conn = Connection::open(db).unwrap();

        let ban_query = "SELECT Count(*) FROM banned WHERE UserId = ?1;";
        let mut is_banned = false;
        {
            let mut ban_stmt = conn.prepare(ban_query).unwrap();

            for row in ban_stmt.query_map(&[("?1", &ctx.author().id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
                let row: u32 = row.unwrap();
                if row == 1 {
                    is_banned = true;
                }
            }
        }

        if is_banned {
            let embed = getembed("Kitiltás",
            "Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.");
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} tiltva van, de megpróbált írni a botnak!", ctx.author().id);

            return Ok(());
        }

        let random_meme = search(&tagek.to_lowercase(), true);

        let description = format!("Itt van egy a leírásnak megfelelő mém: {}", random_meme);
        let mut embed = getembed("Mém megtalálva:", description);
        if random_meme.is_empty() {
            embed = getembed("Nincs ilyen mém.", "A rendszer nem talált a kritériumoknak megfelelő mémet.");
        }
        
        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);

        let reply = CreateReply::new().embed(embed).components(vec![CreateActionRow::Buttons(vec![button])]);

        ctx.send(reply).await.unwrap();
        
        info!("{} lekért egy ilyen random mémet: {}", ctx.author().name, &tagek);

        Ok(())
}

fn search(tagek: &String, random: bool) -> String {
    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).unwrap();
    let tag_vec: Vec<&str> = tagek.split(TAG_SEPARATOR).collect();

    let mut memes_vec: Vec<String> = Vec::new();
    for i in 0..tag_vec.len() {
        let query = "SELECT Memes FROM tags WHERE Tag = ?1";
        let mut stmt = conn.prepare(query).unwrap();
        for row in stmt.query_map(&[("?1", &tag_vec[i])], |row| Ok(row.get(0).unwrap())).unwrap() {
            let memes: String = row.unwrap();
            if i == 0 {
                memes_vec = serde_json::from_str(&memes).unwrap();
            } else {
                let next_memes_vec: Vec<String> = serde_json::from_str(&memes).unwrap();
                memes_vec.retain(|a| next_memes_vec.contains(a));
            }
        }
    }

    if memes_vec.is_empty() {
        return String::new();
    }

    let query = "SELECT Link FROM memes WHERE FileName = ?1;";
            
    let mut stmt = conn.prepare(&query).unwrap();

    if random {
        let mut rng = rand::thread_rng();
        
        memes_vec.shuffle(&mut rng);

        for row in stmt.query_map(&[("?1", &memes_vec[0])], |row| Ok(row.get(0).unwrap())).unwrap() {
            let link: String = row.unwrap();
            return link;
        }
    }

    let mut meme_links: String = String::new();
    for meme in memes_vec {
        if !meme_links.is_empty() {
            meme_links = meme_links + ", ";
        }
        let mut link = String::new();
        for row in stmt.query_map(&[("?1", &meme)], |row| Ok(row.get(0).unwrap())).unwrap() {
            link = row.unwrap();
        }
        meme_links = meme_links + &link;
    }

    meme_links
}