use std::env;
use poise::serenity_prelude::{Color, CreateEmbed, CreateEmbedFooter};

pub fn getembed(title: impl Into<String>, description: impl Into<String>) -> CreateEmbed {
    let title: String = title.into();
    let description: String = description.into();

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    let embed = CreateEmbed::new().color(color)
     .title(title)
     .description(description)
     .footer(CreateEmbedFooter::new(footer_text)
     .icon_url(footer_icon));
    
    embed
}