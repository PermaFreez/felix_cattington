use poise::CreateReply;
use log::info;

use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Kiad egy random mémet az adott tag(ekk)el
#[poise::command(slash_command, dm_only)]
pub async fn search(ctx: Context<'_>,
    #[description = "Kereső tag"] tag: String) -> Result<(), Error> {
        let reply = CreateReply::new().content(format!("WIP ({})", tag));

        ctx.send(reply).await.unwrap();
        
        info!("{} rákeresett erre: {}", ctx.author().name, &tag);

        Ok(())
}