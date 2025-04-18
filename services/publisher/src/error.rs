use std::str::Utf8Error;

use fuel_data_parser::DataParserError;
use fuel_message_broker::MessageBrokerError;
use fuel_streams_core::types::*;
use fuel_streams_domains::{
    infra::repository::RepositoryError,
    MsgPayloadError,
};

#[derive(thiserror::Error, Debug)]
pub enum PublishError {
    #[error("Processing was cancelled")]
    Cancelled,
    #[error(transparent)]
    Db(#[from] fuel_streams_domains::infra::DbError),
    #[error(transparent)]
    FuelCore(#[from] FuelCoreError),
    #[error(transparent)]
    MsgPayload(#[from] MsgPayloadError),
    #[error(transparent)]
    Encoder(#[from] DataParserError),
    #[error(transparent)]
    MessageBrokerClient(#[from] MessageBrokerError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error("Failed to get sealed block from block height")]
    BlockNotFound,
    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),
    #[error("Failed to get block height from encoded utf8 string")]
    BlockHeightFromUtf8(#[from] Utf8Error),
    #[error("Failed to process historical blocks")]
    Historical(String),
}
