use std::{env, fs, path};

use poise::serenity_prelude::*;

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

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");

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

            let mut locked = false;

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
                    // Megnézi repost-e
                    let duplicate = std::str::from_utf8(&out.stdout).unwrap();
                    if duplicate != "" {
                        let query = "SELECT Link FROM memes WHERE FileName = ?1;";

                        let mut link = String::new();
                        {
                            let mut stmt = conn.prepare(query).unwrap();

                            for row in stmt.query_map(&[("?1", duplicate)], |row| Ok(row.get(0).unwrap())).unwrap() {
                                link = row.unwrap();
                            }
                        }

                        let repost_description: String = format!("Úgy tűnik a mém, amit beküldtél már korábban regisztrálva lett: {}. 
                            Amennyiben az időkülönbséget a két beküldés között túl rövidnek ítéled töröld a sajátod.
                            **Amennyiben ezek a mémek NEM egyeznek használd a Fals-pozitív gombot. 
                            A gombbal való visszaélés büntetést von maga után!**", &link);

                        let embed = CreateEmbed::new().color(color)
                            .thumbnail(&attachment.url)
                            .title("Repost észlelve")
                            .description(repost_description)
                            .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));
            
                        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
                        let button2 = CreateButton::new(format!("fals-poz {}", &file)).label("Fals-pozitív").style(ButtonStyle::Danger);

                        msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button).button(button2)).await.unwrap();
                        locked = true;
                    }
                    break;
                } else {
                    suffix+=1;
                }
            }

            let query = "INSERT INTO memes (FileName, Id, Link, Tags, Locked) VALUES (?1, ?2, ?3, ?4, ?5);";
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

            conn.execute(&query, (&file, &msg.id.to_string(), &msg.link(), String::new(), locked)).unwrap();
            conn.execute(&query2, (&msg.author.id.to_string(), &memes_json)).unwrap();

            let description = format!("Úgy tűnik beküldtél egy mémet az Ideológiák Tárháza Discord szerverére. 
                Amennyiben fel szeretnéd venni az IT mém-könyvtárába, használd a `/tag `**`{}`**` <tagek>` parancsot!", &file);

            let embed = CreateEmbed::new().color(color)
                .thumbnail(&attachment.url)
                .title("Mém észlelve")
                .description(description)
                .footer(CreateEmbedFooter::new(&footer_text).icon_url(&footer_icon));

            let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);

            if !locked {
                msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button)).await.unwrap();
            }
        }
    }
}