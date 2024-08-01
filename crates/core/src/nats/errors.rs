use async_nats::{
    error,
    jetstream::{
        context::{CreateStreamErrorKind, GetStreamErrorKind},
        stream::ConsumerErrorKind,
    },
    ConnectErrorKind,
};
use thiserror::Error;

use super::types::PayloadSize;

#[derive(Error, Debug)]
pub enum NatsError {
    #[error("You need to connect before execute the method {0}")]
    ClientNotConnected(&'static str),

    #[error("This client is already connected")]
    ClientAlreadyConnected,

    #[error("{subject:?} payload size={payload_size:?} exceeds max_payload_size={max_payload_size:?}")]
    PayloadTooLarge {
        subject: String,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },

    #[error("Failed to create NATS stream {name}: {source}")]
    CreateStreamFailed {
        name: String,
        #[source]
        source: error::Error<CreateStreamErrorKind>,
    },

    #[error("Failed to find stream with name {name}")]
    GetStreamFailed {
        name: String,
        #[source]
        source: error::Error<GetStreamErrorKind>,
    },

    #[error("Failed to create NATS consumer: {source}")]
    CreateConsumerFailed {
        #[source]
        source: error::Error<ConsumerErrorKind>,
    },

    #[error("Failed to connect to NATS server at {url}: {source}")]
    ConnectionError {
        url: String,
        #[source]
        source: error::Error<ConnectErrorKind>,
    },
}
