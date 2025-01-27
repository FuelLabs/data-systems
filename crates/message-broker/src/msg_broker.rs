use std::fmt;

use async_trait::async_trait;
use futures::Stream;

/// Represents a namespace for message broker subjects/topics
#[derive(Debug, Clone, Default)]
pub enum Namespace {
    Custom(String),
    #[default]
    None,
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Namespace::Custom(s) => write!(f, "{s}"),
            Namespace::None => write!(f, "none"),
        }
    }
}

impl Namespace {
    pub fn subject_name(&self, val: &str) -> String {
        match self {
            Namespace::Custom(s) => format!("{s}.{val}"),
            Namespace::None => val.to_string(),
        }
    }

    pub fn queue_name(&self, val: &str) -> String {
        match self {
            Namespace::Custom(s) => format!("{s}_{val}"),
            Namespace::None => val.to_string(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MessageBrokerError {
    #[error("Failed to connect to broker: {0}")]
    Connection(String),
    #[error("Failed to setup broker infrastructure: {0}")]
    Setup(String),
    #[error("Failed to publish message: {0}")]
    Publishing(String),
    #[error("Failed to receive message: {0}")]
    Receiving(String),
    #[error("Failed to acknowledge message: {0}")]
    Acknowledgment(String),
    #[error("Failed to subscribe: {0}")]
    Subscription(String),
    #[error("Failed to flush: {0}")]
    Flush(String),
    #[error(transparent)]
    Serde(#[from] serde_json::Error),
    #[error(transparent)]
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
    #[error(transparent)]
    NatsSubscribe(#[from] async_nats::client::SubscribeError),
    #[error(transparent)]
    NatsPublish(
        #[from] async_nats::error::Error<async_nats::client::PublishErrorKind>,
    ),
}

#[async_trait]
pub trait Message: std::fmt::Debug + Send + Sync {
    fn payload(&self) -> Vec<u8>;
    async fn ack(&self) -> Result<(), MessageBrokerError>;
    fn id(&self) -> String;
}

pub type MessageBlockStream = Box<
    dyn Stream<Item = Result<Box<dyn Message>, MessageBrokerError>>
        + Send
        + Unpin,
>;

pub type MessageStream = Box<
    dyn Stream<Item = Result<(String, Vec<u8>), MessageBrokerError>>
        + Send
        + Unpin,
>;
