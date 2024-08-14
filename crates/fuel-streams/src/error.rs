use displaydoc::Display as DisplayDoc;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, DisplayDoc)]
pub enum Error {
    /// client error
    ClientError(#[from] crate::client::ClientError),

    /// stream error
    StreamError(#[from] crate::stream::StreamError),
}
