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
use fuel_data_parser::Error as DataParserError;
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

    /// failed to connect to {url}
    ConnectionError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },

    /// data parser error
    DataParser(#[from] DataParserError),

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
