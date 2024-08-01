use streams_core::nats::NatsError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("Failed to connect to URL {url}: {source}")]
    ConnectionError {
        url: String,
        #[source]
        source: NatsError,
    },
}
