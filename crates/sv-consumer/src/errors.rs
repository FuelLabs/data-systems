use fuel_streams_core::StreamError;
use fuel_streams_domains::MsgPayloadError;
use fuel_streams_store::{record::RecordPacketError, store::StoreError};

#[derive(thiserror::Error, Debug)]
pub enum ConsumerError {
    #[error("Failed to start telemetry")]
    TelemetryStart,
    #[error("Failed to start web server")]
    WebServerStart,
    #[error("Processing timed out")]
    Timeout,
    #[error(transparent)]
    Deserialization(#[from] bincode::Error),
    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),
    #[error(transparent)]
    MsgPayload(#[from] MsgPayloadError),
    #[error(transparent)]
    JoinTasks(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Semaphore(#[from] tokio::sync::AcquireError),
    #[error(transparent)]
    Db(#[from] fuel_streams_store::db::DbError),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Stream(#[from] StreamError),
    #[error(transparent)]
    PacketError(#[from] RecordPacketError),
    #[error(transparent)]
    MessageBrokerClient(#[from] fuel_message_broker::MessageBrokerError),
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
}
