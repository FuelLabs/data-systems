use displaydoc::Display as DisplayDoc;
use fuel_streams_core::stream::StreamerError;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum StreamError {
    /// failed to subscribe
    Subscribe {
        #[source]
        source: StreamerError,
    },

    /// failed to subscribe with options
    SubscribeWithOpts {
        #[source]
        source: StreamerError,
    },
}
