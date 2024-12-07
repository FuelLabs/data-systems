use async_nats::{
    error,
    jetstream::{
        consumer::StreamErrorKind,
        context::{CreateKeyValueErrorKind, CreateStreamErrorKind},
        kv::{CreateError, CreateErrorKind, PutError, WatchErrorKind},
        stream::{ConsumerErrorKind, LastRawMessageErrorKind},
    },
};
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Error, DisplayDoc, Debug)]
pub enum StreamError {
    /// Failed to publish to stream: {subject_name}
    PublishFailed {
        subject_name: String,
        #[source]
        source: error::Error<CreateErrorKind>,
    },

    /// Failed to publish to S3
    S3PublishError(#[from] fuel_streams_storage::s3::S3ClientError),

    /// Failed to retrieve last published message from stream
    GetLastPublishedFailed(#[from] error::Error<LastRawMessageErrorKind>),

    /// Failed to create Key-Value Store
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),

    /// Failed to publish item to Key-Value Store
    StorePublish(#[from] PutError),

    /// Failed to subscribe to subject in Key-Value Store
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),

    /// Failed to publish item to stream
    StreamPublish(#[from] CreateError),

    /// Failed to create stream
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),

    /// Failed to create consumer for stream
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),

    /// Failed to consume messages from stream
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
}
