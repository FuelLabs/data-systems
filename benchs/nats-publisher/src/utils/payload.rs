use std::time::{SystemTime, UNIX_EPOCH};

use async_nats::jetstream::context::Publish;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NatsPayload<T>
where
    T: Serialize + Clone,
{
    pub subject: Option<String>,
    pub timestamp: u128,
    #[serde(bound(deserialize = "T: Deserialize<'de>"))]
    pub data: T,
}

impl<T> NatsPayload<T>
where
    T: Serialize + for<'de> Deserialize<'de> + Clone,
{
    pub fn new(data: T) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis();
        Self {
            subject: None,
            data,
            timestamp,
        }
    }

    pub fn with_subject(&mut self, subject: String) -> &mut Self {
        self.subject = Some(subject);
        self
    }

    pub fn serialize(&self) -> Result<Vec<u8>, bincode::Error> {
        bincode::serialize(self)
    }

    pub fn to_publish(&self) -> Result<Publish, bincode::Error> {
        Ok(Publish::build()
            .message_id(self.subject.clone().unwrap())
            .payload(self.serialize()?.into()))
    }

    pub fn from_slice(slice: &[u8]) -> Result<Self, bincode::Error> {
        bincode::deserialize(slice)
    }
}
