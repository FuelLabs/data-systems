use async_nats::jetstream::context::CreateStreamErrorKind;
use async_nats::jetstream::stream::ConsumerErrorKind;
use async_nats::ConnectErrorKind;
use thiserror::Error;

use super::Subject;
use crate::types::PayloadSize;

#[derive(Error, Debug)]
pub enum NatsError {
    #[error("{subject:?} payload size={payload_size:?} exceeds max_payload_size={max_payload_size:?}")]
    PayloadTooLarge {
        subject: Subject,
        payload_size: PayloadSize,
        max_payload_size: PayloadSize,
    },

    #[error("Failed to create NATS stream")]
    CreateStreamFailed {
        #[source]
        source: async_nats::error::Error<CreateStreamErrorKind>,
    },

    #[error("Failed to create NATS consumer")]
    CreateConsumerFailed {
        #[source]
        source: async_nats::error::Error<ConsumerErrorKind>,
    },

    #[error("Failed to connect to NATS server at {url}")]
    ConnectError {
        url: String,
        #[source]
        source: async_nats::error::Error<ConnectErrorKind>,
    },

    #[error("Connection to NATS server at {0} is pending")]
    ConnectionPending(String),
    #[error("Connection to NATS server at {0} is disconnected")]
    ConnectionDisconnected(String),
}
