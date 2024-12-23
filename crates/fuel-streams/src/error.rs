use displaydoc::Display as DisplayDoc;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError, DisplayDoc)]
pub enum Error {
    /// WebSocket client error: {0}
    Client(#[from] crate::client::error::ClientError),
}
