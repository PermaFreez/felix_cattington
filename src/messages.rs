use std::{env, fs};
use log::info;

use poise::serenity_prelude::{async_trait, EventHandler, Context, Message,
    CreateButton, ButtonStyle, CreateMessage
};

use std::process::Command;
use rusqlite::Connection;

use crate::response::getembed;

use crate::{turnoff, schedule, introduce};
pub struct InformerHandler;

#[async_trait]
impl EventHandler for InformerHandler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Megnézi egy mém csatornába érkezett-e az üzenet.
        
        let env_channels = env::var("MEME_CHANNEL").expect("Couldn't find environment variable!");
        let channels: Vec<&str> = env_channels.split(' ').collect();
        let channel_id: &str = &msg.channel_id.to_string();

        if !channels.contains(&channel_id) {
            return;
        }

        for attachment in &msg.attachments {

            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();

            let ban_query = "SELECT Count(*) FROM banned WHERE UserId = ?1;";
            {
                let mut ban_stmt = conn.prepare(ban_query).unwrap();

                for row in ban_stmt.query_map(&[("?1", &msg.author.id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
                    let row: u32 = row.unwrap();
                    if row == 1 {
                        return;
                    }
                }
            }
        
            // Megnézi videó, vagy kép-e a csatolmány (mert hang nyilván nem lehet mém)
            let att_type = &attachment.content_type.to_owned().unwrap();
            if att_type.matches("image").count() == 0 && att_type.matches("video").count() == 0 && att_type.matches("audio").count() == 0 { 
                continue;
            }
            
            // Hozzáad egy -n-t a fájlnévhez, hogy ugyanazzal a fájlnévvel több mémet is lehessen kezelni.
            let mut filename = attachment.filename.clone();
            let mut suffix: u32 = 0;

            if filename.len() > 70 {
                filename = String::from("too.long");
            }

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

                let mut filenamecount: i8 = 0;
                {
                    let query = "SELECT Count(*) FROM memes WHERE FileName = ?1";
                    let mut stmt = conn.prepare(&query).unwrap();
                    for row in stmt.query_map(&[("?1", &file)], |row| Ok(row.get(0).unwrap())).unwrap() {
                        filenamecount = row.unwrap();
                    };
                };

                if filenamecount == 0 {
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

                        let repost_description: String = format!("Úgy tűnik a mém, amit beküldtél már korábban regisztrálva lett: {}. \
                        Amennyiben az időkülönbséget a két beküldés között túl rövidnek ítéled töröld a sajátod. \
                        **Amennyiben ezek a mémek NEM egyeznek használd a Fals-pozitív gombot. \
                        A gombbal való visszaélés büntetést von maga után!**", &link);

                        let mut embed = getembed("Repost észlelve", &repost_description);

                        if att_type.matches("video").count() == 1 {
                            embed = embed.thumbnail("https://cdn.discordapp.com/attachments/873153317939867708/1099754267167961088/iu.png");
                        } else if att_type.matches("audio").count() == 1 {
                            embed = embed.thumbnail("https://cdn.discordapp.com/attachments/873153317939867708/1099755985532366908/iu.png");
                        } else {
                            embed = embed.thumbnail(&attachment.url);
                        }
            
                        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
                        let button2 = CreateButton::new(format!("fals-poz {}", &file)).label("Fals-pozitív").style(ButtonStyle::Danger);

                        msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button).button(button2)).await.unwrap();
                        info!("Repost érzékelve: {}. Küldte: {}.", &file, msg.author.id);
                        locked = true;
                    }
                    break;
                } else {
                    suffix+=1;
                }
            }

            let query = "INSERT INTO memes (FileName, Id, Link, Tags, Locked) VALUES (?1, ?2, ?3, ?4, ?5);";

            conn.execute(&query, (&file, &msg.id.to_string(), &msg.link(), String::new(), locked)).unwrap();
            crate::user::add_ownership(&msg.author.id.to_string(), &file);

            tokio::spawn(schedule::unlock_public(attachment.url.clone(), file.clone(), ctx.clone()));

            introduce::introduce(&msg, &ctx).await;

            if turnoff::is_user_unsubscribed(&msg.author) {
                return;
            }

            let description = format!("Úgy tűnik beküldtél egy mémet az Ideológiák Tárháza Discord szerverére. \
            Amennyiben fel szeretnéd venni az IT mém-könyvtárába, használd a `/tag `**`{}`**` <tagek vesszővel elválasztva>` parancsot!", &file);

            let mut embed = getembed("Mém észlelve", &description);

            if att_type.matches("video").count() == 1 {
                embed = embed.thumbnail("https://cdn.discordapp.com/attachments/873153317939867708/1099754267167961088/iu.png");
            } else if att_type.matches("audio").count() == 1 {
                embed = embed.thumbnail("https://cdn.discordapp.com/attachments/873153317939867708/1099755985532366908/iu.png");
            } else {
                embed = embed.thumbnail(&attachment.url);
            }

            let button = CreateButton::new(format!("quicktag@{}", &file)).label("Gyorscimkézés").style(ButtonStyle::Primary);
            let button2 = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);

            if !locked {
                msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button).button(button2)).await.unwrap();
                info!("Mém érzékelve: {}. Küldő: {}", &file, msg.author.id);
            }
        }
    }
}