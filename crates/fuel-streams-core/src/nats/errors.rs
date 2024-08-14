use async_nats::{
    error,
    jetstream::{
        context::CreateKeyValueErrorKind,
        kv::{PutErrorKind, WatchErrorKind},
        stream::LastRawMessageErrorKind,
    },
    ConnectErrorKind,
};
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Error, DisplayDoc, Debug)]
pub enum NatsError {
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
pub enum StreamError {
    /// failed to publish stream
    PublishFailed {
        subject_name: String,
        #[source]
        source: error::Error<PutErrorKind>,
    },

    /// failed to subscribe to stream
    SubscriptionFailed {
        subject_name: String,
        #[source]
        source: error::Error<WatchErrorKind>,
    },

    /// failed to subscribe to stream
    GetLastPublishedFailed {
        subject_name: String,
        #[source]
        source: error::Error<LastRawMessageErrorKind>,
    },
}
