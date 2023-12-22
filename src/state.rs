use std::collections::{HashMap, VecDeque};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub room_id: String,
    pub user_id: String,
    pub text: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct Messages {
    pub messages: Vec<Message>,
}

pub type RoomStore = HashMap<String, VecDeque<Message>>;

#[derive(Default)]
pub struct MessageStore {
    pub messages: RwLock<RoomStore>,
}

impl MessageStore {
    pub async fn insert(&self, message: Message) {
        let mut guard = self.messages.write().await;
        let messages = guard.entry(message.room_id.clone()).or_default();
        messages.push_front(message);
        messages.truncate(100);
    }

    pub async fn get(&self, room_id: &str) -> Vec<Message> {
        let messages = self.messages.read().await.get(room_id).cloned();
        messages.unwrap_or_default().into_iter().rev().collect()
    }
}
