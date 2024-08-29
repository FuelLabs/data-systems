use async_nats::{
    client::FlushErrorKind,
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
    /// Failed to publish to stream: {subject_name}
    PublishFailed {
        subject_name: String,
        #[source]
        source: error::Error<PutErrorKind>,
    },

    /// Failed to retrieve last published message from stream
    GetLastPublishedFailed(#[from] error::Error<LastRawMessageErrorKind>),

    /// Failed to create Key-Value Store
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),

    /// Failed to publish item to Key-Value Store
    StorePublish(#[from] PutError),

    /// Failed to subscribe to subject in Key-Value Store
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),

    /// Failed to publish item to stream
    StreamPublish(#[from] PublishError),

    /// Failed to create stream
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),

    /// Failed to create consumer for stream
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),

    /// Failed to consume messages from stream
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),

    /// failed to flush messages in the stream
    StreamFlush(#[from] error::Error<FlushErrorKind>),
}
