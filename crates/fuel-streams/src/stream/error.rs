use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum StreamError {
    /// Failed to subscribe to the stream
    Subscribe {
        #[source]
        source: fuel_streams_core::StreamError,
    },

    /// Failed to subscribe to the stream with custom options
    SubscribeWithOpts {
        #[source]
        source: fuel_streams_core::StreamError,
    },
}
