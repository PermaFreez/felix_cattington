use poise::serenity_prelude::{async_trait, EventHandler, Context,
    Guild, Ready, ChannelId, GuildId
};
use log::info;
use std::{env, num::NonZeroU64};

pub struct BelepHandler;

#[async_trait]
impl EventHandler for BelepHandler {
    async fn guild_create(&self, _ctx: Context, guild: Guild, is_new: Option<bool>) {
        if is_new == Some(true) {
            info!("Újonan csatlakozott szerver: {}", guild.name);
        } else if is_new == Some(false) {
            info!("Korábban csatlakozott szerver: {}", guild.name);
        }
    }

    async fn ready(&self, ctx: Context, ready: Ready) {
        let channels = env::var("MEME_CHANNEL").expect("Couldn't find environment variable!");
        let channels_vec: Vec<&str> = channels.split(' ').collect();

        for channel in channels_vec {
            let channel_u64: NonZeroU64 = channel.parse().unwrap();
            let channel_id = ChannelId(channel_u64);
            let channel_struct = &ctx.http.get_channel(channel_id).await.unwrap();

            let guild_id = &channel_struct.clone().guild().unwrap().guild_id.to_string();
            let guild_id_u64: NonZeroU64 = guild_id.parse().unwrap();
            let guild_struct = &ctx.http.get_guild(GuildId(guild_id_u64)).await.unwrap();

            info!("Figyelt csatornák: {}/{}", &guild_struct.name.to_string(), 
                &channel_struct.clone().guild().unwrap().name().to_string())
        }
        
        info!("{} csatlakozva!", ready.user.name);
    }
}