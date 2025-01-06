use async_nats::{
    error,
    jetstream::{
        consumer::StreamErrorKind,
        context::{CreateKeyValueErrorKind, CreateStreamErrorKind},
        kv::{CreateError, CreateErrorKind, PutError, WatchErrorKind},
        stream::{ConsumerErrorKind, LastRawMessageErrorKind},
    },
    PublishError,
    SubscribeError,
};
use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Error, DisplayDoc, Debug)]
pub enum StreamError {
    /// Failed to publish to stream: {subject_name}, error: {source}
    PublishFailed {
        subject_name: String,
        #[source]
        source: error::Error<CreateErrorKind>,
    },
    /// Failed to retrieve last published message from stream: {0}
    GetLastPublishedFailed(#[from] error::Error<LastRawMessageErrorKind>),
    /// Failed to create Key-Value Store: {0}
    StoreCreation(#[from] error::Error<CreateKeyValueErrorKind>),
    /// Failed to publish item to Key-Value Store: {0}
    StorePublish(#[from] PutError),
    /// Failed to subscribe to subject in Key-Value Store: {0}
    StoreSubscribe(#[from] error::Error<WatchErrorKind>),
    /// Failed to publish item to stream: {0}
    StreamPublish(#[from] CreateError),
    /// Failed to create stream: {0}
    StreamCreation(#[from] error::Error<CreateStreamErrorKind>),
    /// Failed to create consumer for stream: {0}
    ConsumerCreate(#[from] error::Error<ConsumerErrorKind>),
    /// Failed to consume messages from stream: {0}
    ConsumerMessages(#[from] error::Error<StreamErrorKind>),
    /// Database error: {0}
    Db(#[from] fuel_streams_store::db::DbError),
    /// Store error: {0}
    Store(#[from] fuel_streams_store::store::StoreError),
    /// Failed to publish to NATS: {0}
    NatsPublish(#[from] PublishError),
    /// Failed to subscribe to NATS: {0}
    NatsSubscribe(#[from] SubscribeError),
}
