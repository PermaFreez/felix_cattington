use std::env;
use serenity::{
    async_trait,
    prelude::*,
    all::{User, Reaction, Message, CreateMessage, CreateEmbed, CreateEmbedFooter, Color},
};
use rusqlite::Connection;
use dotenv::dotenv;

pub struct TurnoffHandler;

#[async_trait]
impl EventHandler for TurnoffHandler {
    async fn reaction_add(&self, ctx: Context, rct: Reaction) {
        let msg = rct.message(&ctx.http).await.unwrap();
        if msg.is_private() && msg.author.bot && rct.emoji.to_string() == "🔴" {
            let user: User = rct.user(&ctx.http).await.unwrap();
            let user_id = user.id.to_string();

            let conn = Connection::open("database.db").expect("Couldn't open databse.");
            let query_on = "INSERT INTO turnoff (UserId) VALUES (?1);";

            if !is_user_unsubscribed(&user) {
                dotenv().ok();
                let footer = CreateEmbedFooter::new(String::from("Készítette: ") + env::var("AUTHOR").expect("Couldn't find AUTHOR environment variable!").as_str())
                    .icon_url("https://cdn.discordapp.com/avatars/418109786622787604/fc8cd6348c2868bc7d3d15ddc0c94ff1.webp");
                let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                    .expect("Color is to be defined in hex!"));
                let description = "A botról sikeresen leiratkoztál! Ezentúl nem fogsz semmilyen üzenetet kapni tőle.
                 Amennyiben vissza szeretnél iratkozni írj bármilyen üzenetet a botnak.";

                conn.execute(query_on, &[("?1", &user_id)]).unwrap();
                let embed = CreateEmbed::new().color(color)
                    .title("Leiratkozva")
                    .description(description)
                    .footer(footer.clone());
                
                user.dm(&ctx.http, CreateMessage::new().embed(embed)).await.unwrap();
            }
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {
        if msg.is_private() {
            let user: User = msg.author;
            let user_id = user.id.to_string();

            let conn = Connection::open("database.db").expect("Couldn't open databse.");
            let query_off = "DELETE FROM turnoff WHERE UserId = ?1;";
            
            if is_user_unsubscribed(&user) {
                dotenv().ok();
                let footer = CreateEmbedFooter::new(String::from("Készítette: ") + env::var("AUTHOR").expect("Couldn't find AUTHOR environment variable!").as_str())
                    .icon_url("https://cdn.discordapp.com/avatars/418109786622787604/fc8cd6348c2868bc7d3d15ddc0c94ff1.webp");
                let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                    .expect("Color is to be defined in hex!"));
                let description = "Sikeresen visszairatkoztál a botra! Újra le tudsz iratkozni a \"🔴\" reakcióval.";

                conn.execute(query_off, &[("?1", &user_id)]).unwrap();
                let embed = CreateEmbed::new().color(color)
                    .title("Visszairatkozva")
                    .description(description)
                    .footer(footer.clone());
                
                user.dm(&ctx.http, CreateMessage::new().embed(embed)).await.unwrap();
            }
        }
    }
}

pub fn is_user_unsubscribed(user: &User) -> bool {
    let conn = Connection::open("database.db").unwrap();
    let query_check = "SELECT Count(*) FROM turnoff  WHERE UserId = ?1;";

    let mut stmt = conn.prepare(&query_check).unwrap();

    for row in stmt.query_map(&[("?1", &user.id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
        let count: i32 = row.unwrap();

        if count == 1 {
            return true;
        } else {
            return false;
        }
    }

    return false;
}