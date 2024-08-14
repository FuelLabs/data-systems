use displaydoc::Display as DisplayDoc;
use fuel_streams_core::nats::NatsError;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum ClientError {
    /// client error
    ConnectionFailed(#[from] NatsError),
}
