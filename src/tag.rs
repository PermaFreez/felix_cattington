use std::env;
use log::info;

use poise::{
    CreateReply,
    serenity_prelude::{UserId, Color, CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle,
        CreateActionRow, Context as Context2, CacheHttp, ChannelId, MessageId}
};

use rusqlite::Connection;

use crate::{Data, TAG_SEPARATOR};
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Tagek beállítása egy adott mémre
#[poise::command(slash_command, dm_only)]
pub async fn tag(ctx: Context<'_>,
    #[description = "Az adott mém fájlneve (feltöltési értesítőben)"] meme: String,
    #[description = "Tagek (Szóközzel elválasztva)"] tagek: String) -> Result<(), Error> {

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    match tag_fn(Some(ctx.clone()), None, &ctx.author().id, &meme, &tagek).await {
        TagResult::Success => {
            let description = format!("Sikeresen beállítottad a következő tageket a *{}* fájlra: **\"{}\"**.", &meme ,&tagek);

            let embed = CreateEmbed::new().color(color)
             .title("Tagek elmentve")
             .description(&description)
             .footer(CreateEmbedFooter::new(footer_text)
             .icon_url(footer_icon));
        
            let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
            let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
            let reply = CreateReply::new().embed(embed).components(components);
        
            ctx.send(reply).await.unwrap();
            info!("{} fájl új tagjei: {}", &meme, &tagek);
        
            Ok(())
        }
        TagResult::Banned => {
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
        TagResult::BannedTag(tag) => {
            let description = format!("A tag-eid között a {}. tiltva van, így a tagek nem frissültek!", tag.1);
            let embed = CreateEmbed::new().color(color)
             .title("Tiltott tag")
             .description(description)
             .footer(CreateEmbedFooter::new(&footer_text)
             .icon_url(&footer_icon));
            let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
            let reply = CreateReply::new().embed(embed).components(vec![CreateActionRow::Buttons(vec![button])]);
            ctx.send(reply).await.unwrap();
            info!("{} megpróbált tiltott szót beállítani tagnek! ({})", ctx.author().id, tag.0);
            return Ok(());
        }
        TagResult::Locked => {
            let embed = CreateEmbed::new().color(color)
             .title("Zárolt mém")
             .description("Ez a mém zárolva van. Ez leggyakrabban amiatt van, mert nem te vagy az első aki beküldte. 
                Amennyiben a mém nem egy repost, a feltöltési értesítő alatt feloldhatod a zárolását.")
             .footer(CreateEmbedFooter::new(footer_text)
             .icon_url(footer_icon));
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} megpróbált egy zárolt mémet tagelni ({})", ctx.author().id, &meme);
            return Ok(());
        }
        TagResult::NotOwned => {
            let embed = CreateEmbed::new().color(color)
             .title("Hiba")
             .description("Ezt a mémet nem te küldted, vagy nem létezik!")
             .footer(CreateEmbedFooter::new(footer_text)
             .icon_url(footer_icon));
            let reply = CreateReply::new().embed(embed);
            ctx.send(reply).await.unwrap();
            info!("{} megpróbált egy nem létező/nem saját mémet tagelni ({})", ctx.author().id, &meme);
            return Ok(());
        }
    }
}

pub enum TagResult {
    Success,
    Banned,
    BannedTag((String, usize)),
    Locked,
    NotOwned,
}

pub async fn tag_fn(ctx1: Option<Context<'_>>, ctx2: Option<Context2>, user: &UserId, filename: &String, tags: &String) -> TagResult {
    let conn = Connection::open("database.db").unwrap();

    if check_banned(user) {
        return TagResult::Banned;
    }

    let tagek_lower = &tags.to_lowercase();

    let tag_split: Vec<&str> = tagek_lower.split(TAG_SEPARATOR).collect::<Vec<&str>>();

    let banned_tags = env::var("BANNED_TAGS").expect("Couldn't find BANNED_TAGS environment variable!");
    let banned_tags_vec: Vec<&str> = banned_tags.split(' ').collect();

    for i in 0..banned_tags_vec.len() {
        if &tags.matches(banned_tags_vec[i]).count() > &0 {
            return TagResult::BannedTag((banned_tags_vec[i].to_string(), &i + 1));
        }
    }

    if !check_ownership(ctx1, ctx2, user, &filename).await {
        return TagResult::NotOwned;
    }

    if check_locked(&filename) {
        return TagResult::Locked;
    }

    let query = "SELECT Tags FROM memes WHERE FileName = ?1;";
    {
        let mut stmt = conn.prepare(&query).unwrap();

        for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
            let tags: String = row.unwrap();

            for tag in tags.split(' ').collect::<Vec<&str>>() {
                let query2 = "SELECT Memes FROM tags WHERE Tag = ?1;";
                let mut stmt2 = conn.prepare(&query2).unwrap();
                for row2 in stmt2.query_map(&[("?1", &tag.trim())], |row| Ok(row.get(0).unwrap())).unwrap() {
                    let memes_json: String = row2.unwrap();
                    let mut memes_vec: Vec<&str> = serde_json::from_str(&memes_json).unwrap();
                    memes_vec.remove(memes_vec.iter().position(|&r| &r == &filename).unwrap());
                    let new_vec = serde_json::to_string(&memes_vec).unwrap();
                    let query3 = "UPDATE tags SET Memes = ?1 WHERE Tag = ?2";

                    conn.execute(&query3, (new_vec, tag)).unwrap();
                }
            }
        }
    }

    let query2 = "UPDATE memes SET Tags = ?1 WHERE FileName = ?2;";

    let mut tags = String::new();
    for tag in 0..tag_split.len() {
        let current_tag = tag_split[tag].trim();

        if tags.is_empty() {
            tags = current_tag.to_string();
        } else {
            tags = tags + " " + current_tag;
        }
        
        let query3 = "SELECT COUNT(*) FROM tags WHERE Tag = ?1;";
        let mut stmt2 = conn.prepare(&query3).unwrap();
        for row2 in stmt2.query_map(&[("?1", &current_tag)], |row| Ok(row.get(0).unwrap())).unwrap() {
            
            let count: i32 = row2.unwrap();

            if count == 0 {

                let mut memes_vec: Vec<&str> = Vec::new();
                memes_vec.push(&filename);
                let new_json = serde_json::to_string(&memes_vec).unwrap();

                let query4 = "INSERT INTO tags (Tag, Memes) VALUES (?1, ?2);";
                conn.execute(&query4, (current_tag, new_json)).unwrap();
                
            } else {
                
                let query4 = "SELECT (Memes) FROM tags WHERE Tag = ?1;";
                let mut stmt3 = conn.prepare(&query4).unwrap();

                for row3 in stmt3.query_map(&[("?1", current_tag)], |row| Ok(row.get(0).unwrap())).unwrap() {

                    let memes: String = row3.unwrap();

                    let mut memes_vec = serde_json::from_str::<Vec<&str>>(&memes).unwrap();
                    if memes_vec.iter().position(|&r| r == filename).is_none() {
                        memes_vec.push(&filename);
                        
                        let new_json = serde_json::to_string(&memes_vec).unwrap();
                        let query5 = "UPDATE tags SET Memes = ?1 WHERE Tag = ?2";
                        conn.execute(&query5, (&new_json, tag_split[tag])).unwrap();
                    }
                }
            }
        }
    }

    conn.execute(&query2, (&tags, &filename)).unwrap();

    return TagResult::Success;
}

pub async fn check_ownership(ctx1: Option<Context<'_>>, ctx2: Option<Context2>, user_id: &UserId, filename: &str) -> bool {
    let conn = Connection::open("database.db").unwrap();
    let mut count: u8 = 0;
    {
        let query = "SELECT Count(*) FROM upforgrabs WHERE FileName = ?1";
        let mut stmt = conn.prepare(&query).unwrap();
        for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
            count = row.unwrap();
        }
    }

    if count == 1 {

        let announce_channel_str = env::var("ANNOUNCE_CHANNEL").expect("Couldn't find ANNOUNCE_CHANNEL environment variable!");
        let announce_channel: u64 = announce_channel_str.parse().unwrap();

        let mut announce_message: u64 = 0;
        {
            let query2 = "SELECT AnnounceMessage FROM upforgrabs WHERE FileName = ?1";
            let mut stmt2 = conn.prepare(&query2).unwrap();
            for row in stmt2.query_map(&[("?1", &filename)], |row| Ok(match row.get(0) {
                Ok(ok) => ok,
                Err(_) => String::from("0"),
            })).unwrap() {
                let announce_message_str: String = row.unwrap();
                announce_message = announce_message_str.parse().unwrap();
            }
        }

        if announce_message != 0 {
            match ctx1 {
                Some(ctx) => {
                    ctx.http().delete_message(ChannelId::new(announce_channel), MessageId::new(announce_message), None).await.unwrap();
                },
                None => {
                    match ctx2 {
                        Some(ctx) => {
                            ctx.http().delete_message(ChannelId::new(announce_channel), MessageId::new(announce_message), None).await.unwrap();
                        },
                        None => (),
                    }
                }
            }
        }

        let query3 = "DELETE FROM upforgrabs WHERE FileName = ?1";
        conn.execute(&query3, &[("?1", &filename)]).unwrap();

        crate::user::add_ownership(&user_id.to_string(), &filename.to_string());
        return true;
    }

    let query2 = "SELECT Memes FROM users WHERE UserId = ?1;";

    let mut stmt2 = conn.prepare(&query2).unwrap();

    for row in stmt2.query_map(&[("?1", &user_id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
        let memes: String = row.unwrap();
        let memes_array: Vec<&str> = serde_json::from_str(&memes).unwrap();
        if memes_array.iter().position(|&r| r == filename).is_some() {
            return true;
        }
    }

    false
}

pub fn check_locked(filename: &str) -> bool {
    let conn = Connection::open("database.db").unwrap();

    let query = "SELECT Locked FROM memes WHERE FileName = ?1;";

    let mut stmt = conn.prepare(&query).unwrap();

    for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
        let locked: bool = row.unwrap();
        
        return locked;
    }

    false
}

pub fn check_banned(user_id: &UserId) -> bool {
    let conn = Connection::open("database.db").unwrap();
    let ban_query = "SELECT Count(*) FROM banned WHERE UserId = ?1;";
    let mut is_banned = false;

    let mut ban_stmt = conn.prepare(ban_query).unwrap();

    for row in ban_stmt.query_map(&[("?1", &user_id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
        let row: u32 = row.unwrap();
        if row == 1 {
            is_banned = true;
        }
    }

    is_banned
}

// Feloldja az összes tageletlen mémet publikus tagelésre
pub fn unlock_all() {
    let conn = Connection::open("database.db").unwrap();

    let query = "SELECT FileName FROM memes WHERE tags = \'\'";
    let query2 = "INSERT INTO upforgrabs (FileName) VALUES (?1)";
    let mut stmt = conn.prepare(&query).unwrap();
    for row in stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap() {
        let filename: String = row.unwrap();

        match conn.execute(&query2, &[("?1", &filename)]) {
            Ok(_) => info!("A \"{}\" mémet mostantól bárki tagelheti!", &filename),
            Err(_) => (),
        }
    }
}