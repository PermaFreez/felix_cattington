pub mod advanced;
pub mod regtemp;

use std::env;
use log::info;

use poise::serenity_prelude::{EventHandler, CreateActionRow, CreateSelectMenu, CreateSelectMenuKind, CreateSelectMenuOption};
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

        if message_component.data.custom_id.matches("quicktag").count() == 1 {

            let user: User = message_component.user.clone();

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let button_id: Vec<&str> = message_component.data.custom_id.split('@').collect();

            let filename = button_id[1];

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

            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();

            let description = format!("Aktiváltad a **`{}`** mém gyorscimkézését! \
            Először válaszd ki a mém formátumát, majd a következő üzenetedben add meg a szereplőjét!", &filename);

            let embed = CreateEmbed::new().color(color)
                 .title("Gyorscimkézés aktiválva")
                 .description(&description)
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));

            let mut menu_options: Vec<CreateSelectMenuOption> = Vec::new();
            {
                let query = "SELECT * FROM templates";
                let mut stmt = conn.prepare(query).unwrap();
                for row in stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap() {
                    let template: String = row.unwrap();
                    menu_options.push(CreateSelectMenuOption::new(&template, &template));
                }
            }
            let drop_down = CreateSelectMenu::new(format!("droptag@{}", &filename),
                CreateSelectMenuKind::String { options: menu_options });
            let select_menu = CreateActionRow::SelectMenu(drop_down);
            let button = CreateButton::new(format!("notemplate@{}", &filename)).label("Nincs listázva").style(ButtonStyle::Primary);
            let buttons = CreateActionRow::Buttons(vec![button]);
                 
            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed)
                .components(vec![select_menu, buttons]));
            message_component.create_response(&ctx.http, reply).await.unwrap();

            info!("{} aktiválta a {} gyorscimkézését!", user_id, &filename);
        }
    }

    async fn message(&self, ctx: Context, msg: Message) {

        if msg.is_private() {
            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();

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

            let mut prev_tags = String::new();
            {
                let query = "SELECT Tags From memes WHERE FileName = ?1";
                let mut stmt = conn.prepare(query).unwrap();
                for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
                    prev_tags = row.unwrap();
                }
            }

            let mut newtags = msg.content.clone();
            if !prev_tags.is_empty() {
                newtags = format!("{}, {}", prev_tags, &msg.content);
            }
            
            println!("{}", newtags);
            match tag::tag_fn(None, Some(ctx.clone()), &msg.author.id, &filename, &newtags).await {
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