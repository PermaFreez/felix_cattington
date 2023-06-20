use std::env;
use log::info;

use poise::serenity_prelude::EventHandler;
use poise::serenity_prelude::{async_trait, Context, Interaction,
    User, CreateEmbed, CreateEmbedFooter,
    Color, CreateInteractionResponse, CreateInteractionResponseMessage,
    ComponentInteractionDataKind
};

use rusqlite::Connection;

use crate::tag;

pub struct AdvancedHandler;

#[async_trait]
impl EventHandler for AdvancedHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {

        let message_component = match intc.message_component() {
            Some(some) => some,
            None => return,
        };

        let droptag = match &message_component.data.kind {
            ComponentInteractionDataKind::StringSelect{values} => {
                match values.get(0) {
                    Some(value) => value.to_owned(),
                    None => return,
                }
            },
            _ => return,
        };
        if message_component.data.custom_id.matches("droptag").count() == 1 {
            let user: User = message_component.user.clone();
            let user_id: String = user.id.to_string();

            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let drop_id: Vec<&str> = message_component.data.custom_id.split('@').collect();

            let filename = drop_id[1];

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
            
            match tag::tag_fn(None, Some(ctx.clone()), &user.id, &filename.to_string(), &droptag.to_string()).await {
                tag::TagResult::Success => {

                    let query = "DELETE FROM quicktag WHERE UserId = ?1;";
                    let query2 = "INSERT INTO quicktag (UserId, FileName) VALUES (?1, ?2);";
        
                    conn.execute(query, &[("?1", &user_id)]).unwrap();
                    conn.execute(query2, (&user_id, &filename)).unwrap();

                    let description = format!("Sikeresen beállítottad a **`{}`** mém formátumát erre: **{}**! \
                    Most a gyorscimkézés második szakasza következik, így \
                    a következő üzeneted összes vesszővel elválasztott része regisztrálva lesz mint cimke!", &filename, &droptag);
        
                    let embed = CreateEmbed::new().color(color)
                         .title("Formátum beállítva")
                         .description(&description)
                         .footer(CreateEmbedFooter::new(&footer_text)
                         .icon_url(&footer_icon));

                    let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
                    message_component.create_response(&ctx.http, reply).await.unwrap();
                    message_component.message.delete(&ctx.http).await.unwrap();
                    info!("{} fájl formátuma: {}", &filename, &droptag);
                }
                _ => (),
            }
        }
    }
}