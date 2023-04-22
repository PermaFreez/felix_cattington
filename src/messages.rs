use std::{env, fs, path};

use serenity::{
    async_trait,
    model::{channel::Message, Color},
    prelude::*, builder::{CreateMessage, CreateEmbed, CreateEmbedFooter},
    all::UserId,
};
use std::process::Command;
use rusqlite::Connection;
use dotenv::dotenv;

use crate::turnoff;
pub struct InformerHandler;

#[async_trait]
impl EventHandler for InformerHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Megnézi a mém csatornába érkezett-e az üzenet.
        if msg.channel_id.to_string() != env::var("MEME_CHANNEL").expect("Couldn't find environment variable!") {
            return;
        }

        for attachment in &msg.attachments {

            if turnoff::is_user_unsubscribed(&msg.author) {
                return;
            }

            dotenv().ok();

            let title: &str = "Mém észlelve";
            let footer = CreateEmbedFooter::new(String::from("Készítette: ") + env::var("AUTHOR").expect("Couldn't find AUTHOR environment variable!").as_str())
                .icon_url("https://cdn.discordapp.com/avatars/418109786622787604/fc8cd6348c2868bc7d3d15ddc0c94ff1.webp");

            let link: &str = "";
            let repost_description: String = String::from("Úgy tűnik a mém, amit beküldtél már korábban regisztrálva lett: ") + link + ". 
                Amennyiben az időkülönbséget a két beküldés között elég rövidnek ítéled töröld a sajátod. 
                **Amennyiben a két mém NEM egyezik, reagálj erre az üzenetre egy \"🔴\" emote-tal!**";
            let repost_continue: &str = "A repost-érzékelést fals-pozitívnak minősítetted. 
                Amennyiben fel szeretnéd venni a mémed az IT mém-könyvtárába, a következő üzenetedben határozz meg szóközzel elválasztott kulcsszavakat!";

            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            
            let conn = Connection::open("database.db").unwrap();
        
            // Megnézi videó, vagy kép-e a csatolmány (mert hang nyilván nem lehet mém)
            let att_type = &attachment.content_type.to_owned().unwrap();
            if att_type.matches("image").count() == 0 && att_type.matches("video").count() == 0 { 
                continue;
            }
            
            // Hozzáad egy -n-t a fájlnévhez, hogy ugyanazzal a fájlnévvel több mémet is lehessen kezelni.
            let filename = &attachment.filename;
            let mut suffix: u32 = 0;

            let filename_parts: Vec<&str> = filename.split('.').collect::<Vec<&str>>();

            let mut filename_last = String::new();

            for i in 1..filename_parts.len() {
                filename_last = filename_last + "." + filename_parts[i];
            }

            let mut file = String::new();
            loop {
                file = format!("{}-{}{}", &filename_parts[0], &suffix.to_string().as_str(), &filename_last);
                let memedir = String::from("memes/");
                fs::create_dir_all(&memedir).unwrap();
                let path: String = memedir + &file;
                if !path::Path::new(&path).exists() {
                    fs::write(&path, &attachment.download().await.unwrap()).expect("Couldn't write to file!");
                    let out = Command::new("python3")
                        .arg("dedup.py")
                        .arg(&file)
                        .output()
                        .expect("Problem with reaching the python component!");
                    let duplicate = std::str::from_utf8(&out.stdout).unwrap();
                    break;
                } else {
                    suffix+=1;
                }
            }

            let query = "INSERT INTO memes (FileName, Link, Tags) VALUES (?1, ?2, ?3);";
            let mut query2 = "INSERT INTO users (UserId, Memes) VALUES (?1, ?2);";

            let query3 = "SELECT Memes FROM users WHERE UserId = ?1";

            let mut memes_json = String::from("[]");

            for memes in conn.prepare(&query3).unwrap().query_map(&[("?1", &msg.author.id.to_string())], |row| Ok(row.get(0))).unwrap() {
                memes_json = memes.unwrap().unwrap();
                query2 = "UPDATE users SET Memes = ?2 WHERE UserId = ?1;";
            }

            let mut memes_vec: Vec<&str> = serde_json::from_str(&memes_json).unwrap();
            memes_vec.push(&file);
            memes_json = serde_json::to_string(&memes_vec).unwrap();

            conn.execute(&query, (&file, &msg.link(), String::new())).unwrap();
            conn.execute(&query2, (&msg.author.id.to_string(), &memes_json)).unwrap();

            let description = format!("Úgy tűnik beküldtél egy mémet az Ideológiák Tárháza Discord szerverére. 
                Amennyiben fel szeretnéd venni az IT mém-könyvtárába, a következő üzenetedben határozz meg szóközzel elválasztott fájlnevet (**{}**) és utána kulcsszavakat! 
                *Amennyiben azt szeretnéd, hogy ez a bot békén hagyjon, reagálj bármelyik üzenetére \"🔴\" emote-tal!*", &file);

            let embed = CreateEmbed::new().color(color)
                .thumbnail(&attachment.url)
                .title(title)
                .description(description)
                .footer(footer.clone());

            msg.author.dm(&ctx.http, CreateMessage::new().embed(embed)).await.unwrap();
        }
    }
}
pub struct TaggingHandler;

#[async_trait]
impl EventHandler for TaggingHandler {
    async fn message(&self, _ctx: Context, msg: Message) {
        if msg.is_private() {

            let conn = Connection::open("database.db").unwrap();

            let message_split = &msg.content.split(' ').collect::<Vec<&str>>();

            let filename = message_split[0];

            if !check_ownership(msg.author.id, filename) {
                return;
            }

            let filename = message_split[0];

            let query = "SELECT Tags FROM memes WHERE FileName = ?1;";

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

            let query2 = "UPDATE memes SET Tags = ?1 WHERE FileName = ?2;";

            let mut tags = String::new();
            for tag in 1..message_split.len() {
                tags = tags + message_split[tag] + " ";
                let query3 = "SELECT COUNT(*) FROM tags WHERE Tag = ?1;";
                let mut stmt2 = conn.prepare(&query3).unwrap();
                for row2 in stmt2.query_map(&[("?1", &message_split[tag])], |row| Ok(row.get(0).unwrap())).unwrap() {
                    
                    let count: i32 = row2.unwrap();

                    if count == 0 {

                        let mut memes_vec: Vec<&str> = Vec::new();
                        memes_vec.push(&filename);
                        let new_json = serde_json::to_string(&memes_vec).unwrap();

                        let query4 = "INSERT INTO tags (Tag, Memes) VALUES (?1, ?2);";
                        conn.execute(&query4, (message_split[tag], new_json)).unwrap();
                        
                    } else {
                        
                        let query4 = "SELECT (Memes) FROM tags WHERE Tag = ?1;";
                        let mut stmt3 = conn.prepare(&query4).unwrap();

                        for row3 in stmt3.query_map(&[("?1", message_split[tag])], |row| Ok(row.get(0).unwrap())).unwrap() {

                            let memes: String = row3.unwrap();

                            let mut memes_vec = serde_json::from_str::<Vec<&str>>(&memes).unwrap();
                            if memes_vec.iter().position(|&r| r == filename).is_none() {
                                memes_vec.push(&filename);
                                
                                let new_json = serde_json::to_string(&memes_vec).unwrap();
                                let query5 = "UPDATE tags SET Memes = ?1 WHERE Tag = ?2";
                                conn.execute(&query5, (&new_json, message_split[tag])).unwrap();
                            }
                        }
                    }
                }
            }
            conn.execute(&query2, (tags, &filename)).unwrap();
        }
    }
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