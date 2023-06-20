use std::{env, fs, io::Write};
use log::info;

use poise::serenity_prelude::{EventHandler, CreateActionRow};
use poise::serenity_prelude::{async_trait, Context, Interaction,
    User, CreateEmbed, CreateEmbedFooter,
    Color, CreateInteractionResponse, CreateInteractionResponseMessage, 
    CreateButton, ButtonStyle
};

use rusqlite::Connection;

pub struct RegTempHandler;

#[async_trait]
impl EventHandler for RegTempHandler {
    async fn interaction_create(&self, ctx: Context, intc: Interaction) {

        let message_component = match intc.message_component() {
            Some(some) => some,
            None => return,
        };

        if message_component.data.custom_id.matches("notemplate").count() == 1 {
            let user: User = message_component.user.clone();
            let user_id = user.id.to_string();
            
            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let button_id: Vec<&str> = message_component.data.custom_id.split('@').collect();

            let filename = button_id[1];

            let description = "Amennyiben a formátum gyakori, de még nem regisztrálható jelentsed az alábbi gommbal! \
            Ezt leszámítva most a gyorscimkézés második szakasza következik, így \
            a következő üzeneted összes vesszővel elválasztott része regisztrálva lesz mint cimke!";

            let embed = CreateEmbed::new().color(color)
                 .title("Formátum nem került regisztrálásra")
                 .description(description)
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));

            let button = CreateButton::new(format!("newtemplate@{}", &filename)).label("Listázás kérése").style(ButtonStyle::Primary);
            let buttons = CreateActionRow::Buttons(vec![button]);
            
            let db = env::var("DATABASE").unwrap();
            let conn = Connection::open(db).unwrap();
            let query = "DELETE FROM quicktag WHERE UserId = ?1;";
            let query2 = "INSERT INTO quicktag (UserId, FileName) VALUES (?1, ?2);";

            conn.execute(query, &[("?1", &user_id)]).unwrap();
            conn.execute(query2, (&user_id, &filename)).unwrap();

            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed)
                 .components(vec![buttons]));
            message_component.create_response(&ctx.http, reply).await.unwrap();
            message_component.message.delete(&ctx.http).await.unwrap();
        }

        if message_component.data.custom_id.matches("newtemplate").count() == 1 {
            let user: User = message_component.user.clone();

            let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
            let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");
        
            let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
                .expect("Color is to be defined in hex!"));

            let button_id: Vec<&str> = message_component.data.custom_id.split('@').collect();

            let filename = button_id[1];

            let mut file = match fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(true)
            .open("newtemplates.txt") {
                Ok(file) => file,
                Err(_) => return,
            };

            let message = format!("{} mém regisztrálása kérelmezve {} által!", &filename, user.id);
            
            write!(file, "\n{}", message).unwrap();

            info!("{}", message);

            let description = format!("Beküldted a {} mémet formátum regisztrálásra! \
            Az adminisztrátor meg fog próbálni nevet adni ennek a típusú mémnek. A következő üzenetre érvényes a gyorscimkézés.", &filename);

            let embed = CreateEmbed::new().color(color)
                 .title("Beküldve regisztrálásra")
                 .description(description)
                 .footer(CreateEmbedFooter::new(&footer_text)
                 .icon_url(&footer_icon));

            let reply = CreateInteractionResponse::Message(CreateInteractionResponseMessage::new().embed(embed));
            message_component.create_response(&ctx.http, reply).await.unwrap();
            message_component.message.delete(&ctx.http).await.unwrap();
        }
    }
}