use poise::serenity_prelude::{ActivityData, model::user::OnlineStatus};
use std::env;

pub fn set_custom_status() -> Option<ActivityData> {
    let activity_type = env::var("ACTIVITY_TYPE").expect("Couldn't find environment variable!");
    let activity_description = env::var("ACTIVITY_DESCRIPTION").expect("Couldn't find environment variable!");

    match activity_type.to_lowercase().as_str() {
        "competing" => Some(ActivityData::competing(activity_description)),
        "listening" => Some(ActivityData::listening(activity_description)),
        "playing" => Some(ActivityData::playing(activity_description)),
        "watching" => Some(ActivityData::watching(activity_description)),
        _ => None
    }
}

pub fn set_online_status() -> OnlineStatus {
    let status = env::var("ONLINE_STATUS").expect("Couldn't find environment variable!");
    match status.to_lowercase().as_str() {
        "donotdisturb" => OnlineStatus::DoNotDisturb,
        "idle" => OnlineStatus::Idle,
        "invisible" => OnlineStatus::Invisible,
        "offline" => OnlineStatus::Offline,
        "online" => OnlineStatus::Online,
        _ => OnlineStatus::Online,
    }
}