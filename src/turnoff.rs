use std::env;
use log::info;

use poise::serenity_prelude::{async_trait, EventHandler, Context, Interaction,
    Color, User, CreateEmbed, CreateEmbedFooter, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateButton, ButtonStyle
};

use rusqlite::Connection;
use dotenv::dotenv;

pub struct TurnoffHandler;

#[async_trait]
impl EventHandler for TurnoffHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {
        let conn = Connection::open("database.db").unwrap();
        dotenv().ok();
        let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
        let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");
        let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
            .expect("Color is to be defined in hex!"));

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
            let embed = CreateEmbed::new().color(color)
            .title("Kitiltás")
            .description("Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.")
            .footer(CreateEmbedFooter::new(&footer_text)
            .icon_url(&footer_icon));
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
                
                let embed = CreateEmbed::new().color(color)
                    .title("Leiratkozva")
                    .description(description)
                    .footer(CreateEmbedFooter::new(footer_text).icon_url(footer_icon));
                
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
                dotenv().ok();
                let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find FOOTER environment variable!");
                let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find FOOTER_ICON environment variable!");
                let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                    .expect("Color is to be defined in hex!"));
                let description = "Sikeresen visszairatkoztál a botra!";

                conn.execute(query_off, &[("?1", &user_id)]).unwrap();
                
                conn.execute(query_off, &[("?1", &user_id)]).unwrap();
                let embed = CreateEmbed::new().color(color)
                    .title("Visszairatkozva")
                    .description(description)
                    .footer(CreateEmbedFooter::new(footer_text).icon_url(footer_icon));

                let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed).button(button));
                
                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} visszairatkozott!", &user.id);
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