use displaydoc::Display as DisplayDoc;
use fuel_streams_core::{nats::NatsError, s3::S3ClientError};
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum ClientError {
    /// Failed to establish connection with the NATS server: {0}
    NatsConnectionFailed(#[from] NatsError),
    /// Failed to establish connection with S3: {0}
    S3ConnectionFailed(#[from] S3ClientError),
}
