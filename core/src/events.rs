use std::sync::Arc;

use serde::{Deserialize, Serialize};
use specta::Type;
use tokio::sync::broadcast;

use crate::Config;

#[derive(Clone, Serialize, Deserialize, Type)]
pub struct Event {
    pub topic: String,
    pub payload: serde_json::Value,
}

#[derive(Clone)]
pub struct EventBus(Arc<broadcast::Sender<Event>>);

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self(Arc::new(tx))
    }

    pub fn publish(&self, topic: impl Into<String>, payload: impl Serialize) {
        let event = Event {
            topic: topic.into(),
            payload: serde_json::to_value(payload).unwrap(),
        };
        let _ = self.0.send(event); // ignore if no subscribers
    }

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.0.subscribe()
    }
}

pub(crate) fn create_event_bus(config: &Config) -> EventBus {
    EventBus::new()
}
