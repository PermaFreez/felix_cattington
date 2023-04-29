use std::env;
use poise::{CreateReply, serenity_prelude::{CreateEmbed, CreateEmbedFooter, CreateButton, ButtonStyle,
    Color, CreateMessage, Context, ChannelId, CreateActionRow}
};
use rusqlite::Connection;
use log::info;

// Elküldi a tagelhetővé vállt mémeket egy publikus csatornára
pub async fn tagging_request(filename: &String, ctx: Context) {
    let conn = Connection::open("database.db").unwrap();

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    let query = "SELECT Link FROM memes WHERE FileName = ?1";
    let mut link = String::new();
    {
        let mut stmt = conn.prepare(&query).unwrap();
        for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
            link = row.unwrap();
        }
    }

    let description = format!("Ezt a mémet még nem cimkézték fel: {}. Legyél te az első, aki tesz ez ellen!\n`/tag {} ...`", &link, &filename);

    let embed = CreateEmbed::new().color(color)
     .title("Új cimkézhető mém")
     .description(&description)
     .footer(CreateEmbedFooter::new(footer_text)
     .icon_url(footer_icon));

    let button = CreateButton::new("leiratkozas").label("Leiratkozás").style(ButtonStyle::Danger);
    let message = CreateMessage::new().embed(embed).button(button);

    let announce_channel: u64 = env::var("ANNOUNCE_CHANNEL").expect("Couldn't find ANNOUNCE_CHANNEL environment variable!").parse().unwrap();

    let channel = ctx.http.get_channel(ChannelId::new(announce_channel)).await.unwrap();
    let announce_message = channel.guild().unwrap().send_message(&ctx.http, message).await.unwrap();

    let query = "UPDATE upforgrabs SET AnnounceMessage = ?1 WHERE FileName = ?2";
    conn.execute(&query, (&announce_message.id.to_string(), &filename)).unwrap();

    info!("A {} fájlnevű mém be lett küldve publikus tagelésre!", &filename);
}

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context2<'a> = poise::Context<'a, Data, Error>;

/// Küld egy olyan mémet, amit cimkézni kell
#[poise::command(slash_command)]
pub async fn cimkezendo(ctx: Context2<'_>) -> Result<(), Error> {

    let conn = Connection::open("database.db").unwrap();

    let footer_text = env::var("FOOTER_TEXT").expect("Couldn't find AUTHOR environment variable!");
    let footer_icon = env::var("FOOTER_ICON").expect("Couldn't find AUTHOR environment variable!");

    let color: Color = Color::new(u32::from_str_radix(env::var("COLOR").expect("Couldn't find environment variable!").as_str(), 16)
        .expect("Color is to be defined in hex!"));

    let query = "SELECT FileName FROM upforgrabs";
    let mut filename = String::new();
    {
        let mut stmt = conn.prepare(&query).unwrap();
        for row in stmt.query_map([], |row| Ok(row.get(0).unwrap())).unwrap() {
            filename = row.unwrap();
            break;
        }
    }

    if filename.is_empty() {
        let description = format!("Sajnos, vagy nem sajnos már minden mém fel van cimkézve.");

        let embed = CreateEmbed::new().color(color)
         .title("Nincs cimkézendő mém")
         .description(&description)
         .footer(CreateEmbedFooter::new(footer_text)
         .icon_url(footer_icon));
    
        let reply = CreateReply::new().embed(embed);
        ctx.send(reply).await.unwrap();
    
        info!("A {} elkért egy cimkézendő mémet, de most nincs ilyen!", &ctx.author().id);
    } else {
        let query2 = "SELECT Link FROM memes WHERE FileName = ?1";
        let mut link = String::new();
        {
            let mut stmt = conn.prepare(&query2).unwrap();
            for row in stmt.query_map(&[("?1", &filename)], |row| Ok(row.get(0).unwrap())).unwrap() {
                link = row.unwrap();
            }
        }

        let description = format!("Ezt a mémet még nem cimkézték fel: {}. `/tag {} ...`", &link, &filename);

        let embed = CreateEmbed::new().color(color)
         .title("Cimkézhető mém")
         .description(&description)
         .footer(CreateEmbedFooter::new(footer_text)
         .icon_url(footer_icon));
    
        let button = CreateButton::new(format!("quicktag@{}", &filename)).label("Gyorscimkézés").style(ButtonStyle::Success);
        let components: Vec<CreateActionRow> = vec![CreateActionRow::Buttons(vec![button])];
        let reply = CreateReply::new().embed(embed).components(components);
        ctx.send(reply).await.unwrap();
    
        info!("A {} megkapta a {} mémet tagelésre!", &ctx.author().id, &filename);
    }

    Ok(())
}