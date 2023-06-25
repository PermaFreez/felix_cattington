use std::env;
use log::info;

use poise::serenity_prelude::{async_trait, EventHandler, Context, Interaction,
    User, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateButton, ButtonStyle
};

use rusqlite::Connection;

use crate::response::getembed;

pub struct TurnoffHandler;

#[async_trait]
impl EventHandler for TurnoffHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {
        let db = env::var("DATABASE").unwrap();
        let conn = Connection::open(db).unwrap();

        let message_component = match intc.message_component() {
            Some(some) => some,
            None => return,
        };

        let user: User = message_component.user.clone();

        let ban_query = "SELECT Count(*) FROM banned WHERE UserId = ?1;";
        let mut is_banned = false;
        {
            let mut ban_stmt = conn.prepare(ban_query).unwrap();

            for row in ban_stmt.query_map(&[("?1", &user.id.to_string())], |row| Ok(row.get(0).unwrap())).unwrap() {
                let row: u32 = row.unwrap();
                if row == 1 {
                    is_banned = true;
                }
            }
        }

        if is_banned {
            let embed = getembed("Kitiltás",
            "Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.");
            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));

            message_component.create_response(&ctx.http, reply).await.unwrap();
            info!("{} tiltva van, de megpróbált írni a botnak!", user.id);

            return;
        }

        if message_component.data.custom_id == "leiratkozas" {
            let user_id = user.id.to_string();

            let query_on = "INSERT INTO turnoff (UserId) VALUES (?1);";

            if !is_user_unsubscribed(&user) {
                let description = "A botról sikeresen leiratkoztál! Ezentúl nem fogsz semmilyen üzenetet kapni tőle.";
                
                conn.execute(query_on, &[("?1", &user_id)]).unwrap();
                
                let embed = getembed("Leiratkozva", description);

                let button = CreateButton::new("visszairatkozas").label("Visszairatkozás").style(ButtonStyle::Success);
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).button(button));

                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} leiratkozott!", &user.id);
            }
        }

        if message_component.data.custom_id == "visszairatkozas" {
            let user_id = user.id.to_string();

            let conn = Connection::open("database.db").expect("Couldn't open databse.");
            let query_off = "DELETE FROM turnoff WHERE UserId = ?1;";
            
            if is_user_unsubscribed(&user) {
                let description = "Sikeresen feliratkoztál a botra!";

                conn.execute(query_off, &[("?1", &user_id)]).unwrap();
                
                conn.execute(query_off, &[("?1", &user_id)]).unwrap();
                let embed = getembed("Feliratkozva", description);

                let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).button(button));
                
                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} visszairatkozott!", &user.id);
            }
        }
    }
}

pub fn is_user_unsubscribed(user: &User) -> bool {
    let db = env::var("DATABASE").unwrap();
    let conn = Connection::open(db).unwrap();
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