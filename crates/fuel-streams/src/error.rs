use displaydoc::Display as DisplayDoc;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, DisplayDoc)]
pub enum Error {
    /// An error occurred in the client
    ClientError(#[from] crate::client::ClientError),

    /// An error occurred in the stream
    StreamError(#[from] crate::stream::StreamError),

    /// Consuming messages error
    MessagesError(#[from] crate::core::nats::types::MessagesError),
}
