use std::env;
use dotenv::dotenv;

use poise::{
    CreateReply,
    serenity_prelude::{UserId, Color, CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle, CreateActionRow},
};

use rusqlite::Connection;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;


/// Tagek beállítása egy adott mémre
#[poise::command(slash_command, dm_only)]
pub async fn tag(ctx: Context<'_>,
    #[description = "Az adott mém fájlneve (feltöltési értesítőben)"] meme: String,
    #[description = "Tagek (Szóközzel elválasztva)"] tagek: String) -> Result<(), Error> {
    
    let conn = Connection::open("database.db").unwrap();

    let tag_split = &tagek.split(' ').collect::<Vec<&str>>();

    let filename = meme;

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    if !check_ownership(ctx.author().id, &filename) {
        let embed = CreateEmbed::new().color(color)
         .title("Hiba")
         .description("Ezt a mémet nem te küldted, vagy nem létezik!")
         .footer(CreateEmbedFooter::new(footer_text)
         .icon_url(footer_icon));
        let reply = CreateReply::new().embed(embed);
        ctx.send(reply).await.unwrap();
        return Ok(());
    }

    if check_locked(&filename) {
        let embed = CreateEmbed::new().color(color)
         .title("Zárolt mém")
         .description("Ez a mém zárolva van. Ez leggyakrabban amiatt van, mert nem te vagy az első aki beküldte. 
            Amennyiben a mém nem egy repost, a feltöltési értesítő alatt feloldhatod a zárolását.")
         .footer(CreateEmbedFooter::new(footer_text)
         .icon_url(footer_icon));
        let reply = CreateReply::new().embed(embed);
        ctx.send(reply).await.unwrap();
        return Ok(());
    }

    dotenv().ok();

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
        tags = tags + tag_split[tag] + " ";
        let query3 = "SELECT COUNT(*) FROM tags WHERE Tag = ?1;";
        let mut stmt2 = conn.prepare(&query3).unwrap();
        for row2 in stmt2.query_map(&[("?1", &tag_split[tag])], |row| Ok(row.get(0).unwrap())).unwrap() {
            
            let count: i32 = row2.unwrap();

            if count == 0 {

                let mut memes_vec: Vec<&str> = Vec::new();
                memes_vec.push(&filename);
                let new_json = serde_json::to_string(&memes_vec).unwrap();

                let query4 = "INSERT INTO tags (Tag, Memes) VALUES (?1, ?2);";
                conn.execute(&query4, (tag_split[tag], new_json)).unwrap();
                
            } else {
                
                let query4 = "SELECT (Memes) FROM tags WHERE Tag = ?1;";
                let mut stmt3 = conn.prepare(&query4).unwrap();

                for row3 in stmt3.query_map(&[("?1", tag_split[tag])], |row| Ok(row.get(0).unwrap())).unwrap() {

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

    let description = format!("Sikeresen beállítottad a következő tageket a *{}* fájlra: **\"{}\"**.", &filename ,&tags);

    let embed = CreateEmbed::new().color(color)
     .title("Tagek elmentve")
     .description(&description)
     .footer(CreateEmbedFooter::new(footer_text)
     .icon_url(footer_icon));

    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
    let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
    let reply = CreateReply::new().embed(embed).components(components);

    ctx.send(reply).await.unwrap();

    Ok(())
}

fn check_ownership(user_id: UserId, filename: &str) -> bool {
    let conn = Connection::open("database.db").unwrap();

    let query = "SELECT Memes FROM users WHERE UserId = ?1;";

    let mut stmt = conn.prepare(&query).unwrap();

    for row in stmt.query_map(&[("?1", &user_id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
        let memes: String = row.unwrap();
        let memes_array: Vec<&str> = serde_json::from_str(&memes).unwrap();
        if memes_array.iter().position(|&r| r == filename).is_some() {
            return true;
        }
    }

    false
}

fn check_locked(filename: &str) -> bool {
    let conn = Connection::open("database.db").unwrap();

    let query = "SELECT Locked FROM memes WHERE FileName = ?1;";

    let mut stmt = conn.prepare(&query).unwrap();

    for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
        let locked: bool = row.unwrap();
        
        return locked;
    }

    false
}