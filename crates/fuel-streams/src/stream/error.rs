use displaydoc::Display as DisplayDoc;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum StreamError {
    /// failed to subscribe
    Subscribe {
        #[source]
        source: fuel_streams_core::StreamError,
    },

    /// failed to subscribe with options
    SubscribeWithOpts {
        #[source]
        source: fuel_streams_core::StreamError,
    },
}
