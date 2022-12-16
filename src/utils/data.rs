use std::{
    fs::{self, File},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use serenity::{all::ChannelId, prelude::*};

const PATH: &str = "saved_data.json";

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Data {
    hackathon_channel: Option<ChannelId>,
}

impl Data {
    pub async fn get_lock(ctx: &Context) -> Arc<RwLock<Data>> {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Data>()
            .expect("Expected EventsCounter in data.")
            .clone()
    }
    pub fn write_to_file(&self) {
        let data = serde_json::to_string_pretty(&self).expect("Serialization failed.");
        fs::write(PATH, data).expect("Can't save data.");
    }
    pub async fn get_hackathon_channel(ctx: &Context) -> Option<ChannelId> {
        let lock = Data::get_lock(ctx).await;
        let read = lock.read().await;
        read.hackathon_channel
    }
    pub async fn edit_hackathon_channel(ctx: &Context, new_hackathon_channel: ChannelId) {
        let lock = Data::get_lock(ctx).await;
        let mut read = lock.write().await;
        read.hackathon_channel = Some(new_hackathon_channel);
        read.write_to_file();
    }
    pub fn from_file() -> Data {
        match File::open(PATH) {
            Err(_) => Data::default(),
            Ok(file) => {
                let reader = std::io::BufReader::new(file);
                match serde_json::from_reader(reader) {
                    Err(_) => Data::default(),
                    Ok(events) => events,
                }
            }
        }
    }
}

impl TypeMapKey for Data {
    type Value = Arc<RwLock<Data>>;
}
