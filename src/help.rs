use std::env;
use log::info;

use poise::{CreateReply, serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle, CreateActionRow}};

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Elmagyarázza a botot
#[poise::command(slash_command, dm_only)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
        
        let description = "**A bot jelenleg három parancsot támogat:**
        ```\
        /tag <fájlnév> <cimkék vesszővel elválasztva>\n\
        Pl. /tag valamivalami-42.png cica, macska, szisza\n\
        Ez rá fogja tenni a valamivalami-42.png mémre a \"cica\" \"macska\" és \"szisza\" cimkét.\
        ```\
        ```\
        /search_all <cimkék>\n\
        Pl. /search_all szisza\n\
        Ez ki fogja adni a valamivalami-42.png-t és minden más \"sziszával\" felcimkézett mémet.\
        ```\
        ```\
        /search_random <cimkék>\n\
        Pl. /search_random \"cica\"\n\
        Ez ki fogja adni a valamivalami-42.png-t, vagy egy másik mémet, amit \"cicával\" cimkéztek fel.\
        ```\
        ```\
        /cimkezendo\n\
        Ez a parancs elküld neked egy olyan mémet, amit a tulajdonosa nem cimkézett fel. Ezt a mémet te is fel tudod cimkézni.\
        ```\
        ```\
        /mosttagged\n\
        Ez a parancs kiadja a 10 leggyakrabban használt cimkét.\
        ```\
        ```\
        /alltagged\n\
        Ez a parancs kiadja az összes eddig használt cimkét.\
        ```";

        let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
        let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

        let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
            .expect("Color is to be defined in hex!"));


        let embed = CreateEmbed::new().color(color)
         .title("Súgó")
         .description(description)
         .footer(CreateEmbedFooter::new(footer_text)
         .icon_url(footer_icon));

        let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
        let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
        let reply = CreateReply::new().embed(embed).components(components);
    
        ctx.send(reply).await.unwrap();
        info!("{} használta a /help parancsot.", &ctx.author().id);

        Ok(())
}