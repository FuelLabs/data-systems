use async_nats::{
    error,
    jetstream::{
        consumer::StreamErrorKind,
        context::{CreateKeyValueErrorKind, CreateStreamErrorKind},
        kv::{PutError, PutErrorKind, WatchErrorKind},
        stream::{ConsumerErrorKind, LastRawMessageErrorKind},
    },
    PublishError,
};
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Error, DisplayDoc, Debug)]
pub enum StreamError {
    /// failed to publish stream
    PublishFailed {
        subject_name: String,
        #[source]
        source: error::Error<PutErrorKind>,
    },

    /// failed to subscribe to stream
    GetLastPublishedFailed(#[from] error::Error<LastRawMessageErrorKind>),

    /// failed to create KV Store
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),

    /// failed to publish item
    StorePublish(#[from] PutError),

    /// failed to subscribe to subject
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),

    /// failed to publish item
    StreamPublish(#[from] PublishError),

    /// failed to create stream
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),

    /// failed to create consumer
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),

    /// failed to consume messages from stream
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
}
