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
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

use super::types::PayloadSize;

#[derive(Error, DisplayDoc, Debug)]
pub enum NatsError {
    /// Payload size exceeds maximum allowed: subject '{subject_name}' has size {payload_size} which is larger than the maximum of {max_payload_size}
    PayloadTooLarge {
        subject_name: String,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },

    /// Failed to connect to NATS server at {url}
    ConnectionError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },

    /// Failed to create Key-Value Store in NATS
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),

    /// Failed to publish item to Key-Value Store
    StorePublish(#[from] PutError),

    /// Failed to subscribe to subject in Key-Value Store
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),

    /// Failed to publish item to NATS stream
    StreamPublish(#[from] PublishError),

    /// Failed to create NATS stream
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),

    /// Failed to create consumer for NATS stream
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),

    /// Failed to consume messages from NATS stream
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
}
