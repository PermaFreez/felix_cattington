use std::env;

use poise::{CreateReply, serenity_prelude::{CreateEmbed, CreateEmbedFooter, Color, CreateButton, ButtonStyle, CreateActionRow}};
use log::info;

use rand::prelude::*;
use rusqlite::Connection;

use crate::{Data, TAG_SEPARATOR};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Kiad egy random mémet az adott tag(ekk)el
#[poise::command(slash_command, dm_only)]
pub async fn search_all(ctx: Context<'_>,
    #[description = "Kereső tagek"] tagek: String) -> Result<(), Error> {
        let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
        let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");

        let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
            .expect("Color is to be defined in hex!"));

            let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

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
            let embed = CreateEmbed::new().color(color)
            .title("Kitiltás")
            .description("Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.")
            .footer(CreateEmbedFooter::new(&footer_text)
            .icon_url(&footer_icon));
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} tiltva van, de megpróbált írni a botnak!", ctx.author().id);

            return Ok(());
        }

        let meme_links = search(&tagek.to_lowercase(), false);

        let mut embed = CreateEmbed::new();
        if meme_links.is_empty() {
            embed = embed
                .title("Nincs ilyen mém.")
                .description("A rendszer nem talált a kritériumoknak megfelelő mémet.")
                .color(color)
                .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));
        } else {
            let description = format!("Itt vannak a leírásnak megfelelő mémek: {}", meme_links);
            embed = embed
                .title("Mém megtalálva:")
                .description(description)
                .color(color)
                .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));
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
        let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
        let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");

        let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
            .expect("Color is to be defined in hex!"));

        let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

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
            let embed = CreateEmbed::new().color(color)
            .title("Kitiltás")
            .description("Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.")
            .footer(CreateEmbedFooter::new(&footer_text)
            .icon_url(&footer_icon));
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} tiltva van, de megpróbált írni a botnak!", ctx.author().id);

            return Ok(());
        }

        let random_meme = search(&tagek.to_lowercase(), true);

        let mut embed = CreateEmbed::new();
        if random_meme.is_empty() {
            embed = embed
                .title("Nincs ilyen mém.")
                .description("A rendszer nem talált a kritériumoknak megfelelő mémet.")
                .color(color)
                .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));
        } else {
            let description = format!("Itt van egy a leírásnak megfelelő mém: {}", random_meme);
            embed = embed
                .title("Mém megtalálva:")
                .description(description)
                .color(color)
                .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));
        }
        
        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);

        let reply = CreateReply::new().embed(embed).components(vec![CreateActionRow::Buttons(vec![button])]);

        ctx.send(reply).await.unwrap();
        
        info!("{} lekért egy ilyen random mémet: {}", ctx.author().name, &tagek);

        Ok(())
}

fn search(tagek: &String, random: bool) -> String {
    let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();
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