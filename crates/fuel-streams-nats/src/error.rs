use async_nats::{
    client::PublishErrorKind,
    error,
    jetstream::{
        consumer::StreamErrorKind,
        context::{self, CreateStreamErrorKind},
        stream::ConsumerErrorKind,
    },
    ConnectErrorKind,
    SubscribeError,
};

use super::types::PayloadSize;

#[derive(thiserror::Error, Debug)]
pub enum NatsError {
    #[error("Payload size exceeds maximum allowed: subject '{subject_name}' has size {payload_size} which is larger than the maximum of {max_payload_size}")]
    PayloadTooLarge {
        subject_name: String,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },
    #[error("Failed to connect to NATS server at {url}")]
    ConnectionError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },
    #[error(transparent)]
    Publish(#[from] error::Error<PublishErrorKind>),
    #[error(transparent)]
    Subscribe(#[from] SubscribeError),
    #[error(transparent)]
    StreamPublish(#[from] error::Error<context::PublishErrorKind>),
    #[error(transparent)]
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),
    #[error(transparent)]
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),
    #[error(transparent)]
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
}
