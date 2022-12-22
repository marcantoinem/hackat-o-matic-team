use super::constants::DATA_PATH;
use serde::{Deserialize, Serialize};
use serenity::{all::ChannelId, prelude::*};
use std::{
    fs::{self, File},
    sync::Arc,
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Preference {
    hackathon_channel: Option<ChannelId>,
    hackathon_category: Option<ChannelId>,
}

impl Preference {
    pub async fn get_lock(ctx: &Context) -> Arc<RwLock<Preference>> {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Preference>()
            .expect("Expected EventsCounter in data.")
            .clone()
    }
    pub fn write_to_file(&self) {
        let data = serde_json::to_string_pretty(&self).expect("Serialization failed.");
        fs::write(DATA_PATH, data).expect("Can't save data.");
    }
    pub async fn get_hackathon_channel(ctx: &Context) -> Option<ChannelId> {
        let lock = Preference::get_lock(ctx).await;
        let read = lock.read().await;
        read.hackathon_channel
    }
    pub async fn edit_hackathon_channel(ctx: &Context, new_hackathon_channel: ChannelId) {
        let lock = Preference::get_lock(ctx).await;
        let mut read = lock.write().await;
        read.hackathon_channel = Some(new_hackathon_channel);
        read.write_to_file();
    }
    pub async fn get_hackathon_category(ctx: &Context) -> Option<ChannelId> {
        let lock = Preference::get_lock(ctx).await;
        let read = lock.read().await;
        read.hackathon_category
    }
    pub async fn edit_hackathon_category(ctx: &Context, new_hackathon_category: ChannelId) {
        let lock = Preference::get_lock(ctx).await;
        let mut read = lock.write().await;
        read.hackathon_category = Some(new_hackathon_category);
        read.write_to_file();
    }
    pub fn from_file() -> Preference {
        match File::open(DATA_PATH) {
            Err(_) => Preference::default(),
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Err(_) => Preference::default(),
                    Ok(events) => events,
                }
            }
        }
    }
}

impl TypeMapKey for Preference {
    type Value = Arc<RwLock<Preference>>;
}