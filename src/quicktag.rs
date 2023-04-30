use std::env;
use log::info;

use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::{async_trait, Context, Interaction,
    User, CreateEmbed, CreateEmbedFooter, Message,
    Color, CreateInteractionResponse, CreateInteractionResponseMessage, 
    CreateButton, ButtonStyle, CreateMessage
};

use rusqlite::{Connection, params};

use crate::tag;

pub struct QuickTagHandler;

#[async_trait]
impl EventHandler for QuickTagHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {

        let message_component = match intc.message_component() {
            Some(some) => some,
            None => return,
        };

        let user: User = message_component.user.clone();

        if message_component.data.custom_id.matches("quicktag").count() == 1 {
            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let button_id: Vec<&str> = message_component.data.custom_id.split('@').collect();

            let filename= button_id[1];

            let user_id = user.id.to_string();

            if tag::check_banned(&message_component.user.id) {
                let embed = CreateEmbed::new().color(color)
                .title("Kitiltás")
                .description("Le vagy tiltva a bot használatáról, amennyiben kérdéseid vannak írj <@418109786622787604>-nak.")
                .footer(CreateEmbedFooter::new(&footer_text)
                .icon_url(&footer_icon));
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} tiltva van, de megpróbált írni a botnak!", user_id);
                return;
            }
            
            if !tag::check_ownership(None, Some(ctx.clone()), &user.id, &filename).await {
                let embed = CreateEmbed::new().color(color)
                 .title("Hiba")
                 .description("Ezt a mémet nem te küldted, vagy nem létezik!")
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} megpróbált egy nem létező/nem saját mémet tagelni ({})", user_id, &filename);
                return;
            }

            if tag::check_locked(&filename) {
                let embed = CreateEmbed::new().color(color)
                 .title("Zárolt mém")
                 .description("Ez a mém zárolva van. Ez leggyakrabban amiatt van, mert nem te vagy az első aki beküldte. 
                    Amennyiben a mém nem egy repost, a feltöltési értesítő alatt feloldhatod a zárolását.")
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));
                let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
                message_component.create_response(&ctx.http, reply).await.unwrap();
                info!("{} megpróbált egy zárolt mémet tagelni ({})", user_id, &filename);
                return;
            }

            let description = format!("Aktiváltad a **`{}`** mém gyorscimkézését! \
            A következő üzeneted összes szava regisztrálva lesz, mint tag!", &filename);

            let embed = CreateEmbed::new().color(color)
                 .title("Gyorscimkézés aktiválva")
                 .description(&description)
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));
            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
            message_component.create_response(&ctx.http, reply).await.unwrap();

            let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

            let query = "DELETE FROM quicktag WHERE UserId = ?1;";
            let query2 = "INSERT INTO quicktag (UserId, FileName) VALUES (?1, ?2);";

            conn.execute(query, &[("?1", &user_id)]).unwrap();
            conn.execute(query2, (&user_id, &filename)).unwrap();

            info!("{} aktiválta a {} gyorscimkézését!", user_id, &filename);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {

        if msg.is_private() {
            let conn = Connection::open(env::var("DATABASE").unwrap()).unwrap();

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let mut filename = String::new();
            {
                let query = "SELECT FileName FROM quicktag WHERE UserId = ?1;";
                let mut stmt = conn.prepare(query).unwrap();
                for row in stmt.query_map(&[("?1", msg.author.id.to_string().as_str())], |row| Ok(row.get(0).unwrap())).unwrap() {
                    filename = row.unwrap();
                }
            }

            if filename.is_empty() {
                return;
            }

            match tag::tag_fn(None, Some(ctx.clone()), &msg.author.id, &filename, &msg.content).await {
                tag::TagResult::Success => {

                    let query = "DELETE FROM quicktag WHERE UserId = ?1;";

                    conn.execute(query, params![msg.author.id.to_string().as_str()]).unwrap();

                    let description = format!("Sikeresen beállítottad a következő tageket a *{}* fájlra: **\"{}\"**.", &filename, &msg.content);


                    let embed = CreateEmbed::new().color(color)
                    .title("Tagek elmentve")
                    .description(&description)
                    .footer(CreateEmbedFooter::new(footer_text)
                    .icon_url(footer_icon));
                    
                    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
                        
                    msg.author.dm(&ctx.http, CreateMessage::new().embed(embed).button(button)).await.unwrap();
                    info!("{} fájl új tagjei: {}", &filename, &msg.content);
                }
                _ => ()
            }
        }
    }
}