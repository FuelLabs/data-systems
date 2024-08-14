use displaydoc::Display as DisplayDoc;
use fuel_streams_core::nats::NatsError;
use thiserror::Error;

#[derive(Debug, Error, DisplayDoc)]
pub enum StreamError {
    /// failed to get or create internal stream
    GetOrInitStream {
        #[source]
        source: NatsError,
    },

    /// failed to subscribe
    Subscribe {
        #[source]
        source: NatsError,
    },

    /// failed to subscribe with options
    SubscribeWithOpts {
        #[source]
        source: NatsError,
    },
}
