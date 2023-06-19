mod helprender;
use helprender::*;

use std::env;
use log::info;

use poise::{CreateReply, serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle, CreateActionRow}};

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Elmagyarázza a botot
#[poise::command(slash_command, dm_only)]
pub async fn help(ctx: Context<'_>) -> Result<(), Error> {
        
        let help = Help::new()
            .add(Command::new("/tag", Some(vec!["fájlnév", "cimkék vesszővel elválasztva"]),
                Some("Pl. /tag valamivalami-42.png cica, macska, szisza"), "Ez rá fogja tenni a valamivalami-42.png mémre a \"cica\" \"macska\" és \"szisza\" cimkét."))
            .add(Command::new("/seach_all", Some(vec!["cimkék"]),
                Some("Pl. /search_all szisza"), "Ez ki fogja adni a valamivalami-42.png-t és minden más \"sziszával\" felcimkézett mémet."))
            .add(Command::new("/search_random", Some(vec!["cimkék"]), Some("Pl. /search_random \"cica\""), "Ez ki fogja adni a valamivalami-42.png-t, vagy egy másik mémet, amit \"cicával\" cimkéztek fel."))
            .add(Command::new("/cimkezendo", None, None,
                "Ez a parancs elküld neked egy olyan mémet, amit a tulajdonosa nem cimkézett fel. Ezt a mémet te is fel tudod cimkézni."))
            .add(Command::new("/mosttagged", None, None, "Ez a parancs kiadja a 10 leggyakrabban használt cimkét."))
            .add(Command::new("/alltagged", None, None, "Ez a parancs kiadja az összes eddig használt cimkét."))
            .add(Command::new("/gettags", Some(vec!["fájlnév"]), None, "Ez a parancs kiadja az adott mémhez tartozó összes cimkét."));

        let description = help.render();

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