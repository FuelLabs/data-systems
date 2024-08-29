use displaydoc::Display as DisplayDoc;
use fuel_streams_core::nats::NatsError;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum ClientError {
    /// Failed to establish connection with the NATS server: {0}
    ConnectionFailed(#[from] NatsError),
}
