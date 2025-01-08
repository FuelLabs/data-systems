use async_nats::{
    error,
    jetstream::{
        consumer::StreamErrorKind,
        context::{
            CreateKeyValueErrorKind,
            CreateStreamErrorKind,
            PublishError,
        },
        kv::{PutError, WatchErrorKind},
        stream::ConsumerErrorKind,
    },
    ConnectErrorKind,
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
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),
    #[error(transparent)]
    StorePublish(#[from] PutError),
    #[error(transparent)]
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),
    #[error(transparent)]
    StreamPublish(#[from] PublishError),
    #[error(transparent)]
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),
    #[error(transparent)]
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),
    #[error(transparent)]
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
}
