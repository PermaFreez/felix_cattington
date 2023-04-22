use crate::Data;
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

/// Ad egy random mémet az adott taggel.
#[poise::command(slash_command, dm_only)]
pub async fn search(ctx: Context<'_>,
    #[description = "Kereső tag"] tag: String) -> Result<(), Error> {
    // Command code here

    Ok(())
}