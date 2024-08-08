use async_nats::{
    error,
    jetstream::{
        context::CreateKeyValueErrorKind,
        kv::{PutErrorKind, WatchErrorKind},
        stream::ConsumerErrorKind,
    },
    ConnectErrorKind,
};
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

use super::types::PayloadSize;

#[derive(Error, DisplayDoc, Debug)]
pub enum NatsError {
    /// {subject_name:?} payload size={payload_size:?} exceeds max_payload_size={max_payload_size:?}
    PayloadTooLarge {
        subject_name: String,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },

    /// failed to create KV Store with name {name}
    CreateStoreFailed {
        name: String,
        #[source]
        source: error::Error<CreateKeyValueErrorKind>,
    },

    /// failed to connect to {url}
    ConnectionError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },
}

#[derive(Error, DisplayDoc, Debug)]
pub enum StoreError {
    /// failed to serialize/deserialize store item
    SerializationFailed(#[from] bincode::Error),

    /// failed to upsert item {key}
    UpsertFailed {
        key: String,
        #[source]
        source: error::Error<PutErrorKind>,
    },

    /// failed to watch all on store {store:?}
    WatchAllFailed {
        store: String,
        #[source]
        source: error::Error<WatchErrorKind>,
    },

    /// failed to watch subject {subject:?}
    WatchFailed {
        subject: String,
        #[source]
        source: error::Error<WatchErrorKind>,
    },

    /// failed to create consumer
    CreateConsumerFailed(#[from] error::Error<ConsumerErrorKind>),
}
