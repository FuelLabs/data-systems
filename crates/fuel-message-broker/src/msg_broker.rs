use std::{fmt, sync::Arc};

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
    Other(#[from] Box<dyn std::error::Error + Send + Sync>),
}

#[async_trait]
pub trait Message: std::fmt::Debug + Send + Sync {
    fn payload(&self) -> Vec<u8>;
    async fn ack(&self) -> Result<(), MessageBrokerError>;
}

pub type MessageBlockStream = Box<
    dyn Stream<Item = Result<Box<dyn Message>, MessageBrokerError>>
        + Send
        + Unpin,
>;

pub type MessageStream =
    Box<dyn Stream<Item = Result<Vec<u8>, MessageBrokerError>> + Send + Unpin>;

#[async_trait]
pub trait MessageBroker: std::fmt::Debug + Send + Sync + 'static {
    /// Get the current namespace
    fn namespace(&self) -> &Namespace;

    /// Setup required infrastructure (queues, exchanges, etc)
    async fn setup(&self) -> Result<(), MessageBrokerError>;

    /// Check if the broker is connected
    fn is_connected(&self) -> bool;

    /// Publish a block to the work queue for processing
    /// Used by publisher to send blocks to consumers
    async fn publish_block(
        &self,
        id: String,
        payload: Vec<u8>,
    ) -> Result<(), MessageBrokerError>;

    /// Receive a stream of blocks from the work queue
    /// Used by consumer to process blocks
    async fn receive_blocks_stream(
        &self,
        batch_size: usize,
    ) -> Result<MessageBlockStream, MessageBrokerError>;

    /// Publish an event to a topic for subscribers
    /// Used by Stream implementation for pub/sub
    async fn publish_event(
        &self,
        topic: &str,
        payload: bytes::Bytes,
    ) -> Result<(), MessageBrokerError>;

    /// Subscribe to events on a topic
    /// Used by Stream implementation for pub/sub
    async fn subscribe_to_events(
        &self,
        topic: &str,
    ) -> Result<MessageStream, MessageBrokerError>;

    /// Flush all in-flight messages
    async fn flush(&self) -> Result<(), MessageBrokerError>;

    /// Check if the broker is healthy
    async fn is_healthy(&self) -> bool;

    /// Get health info
    async fn get_health_info(&self) -> serde_json::Value;
}

#[derive(Debug, Clone, Default)]
pub enum MessageBrokerClient {
    #[default]
    Nats,
}

impl MessageBrokerClient {
    pub async fn start(
        &self,
        url: &str,
    ) -> Result<Arc<dyn MessageBroker>, MessageBrokerError> {
        match self {
            MessageBrokerClient::Nats => {
                let opts = crate::NatsOpts::new(url.to_string());
                let broker = crate::NatsMessageBroker::new(&opts).await?;
                broker.setup().await?;
                Ok(broker.arc())
            }
        }
    }
}
