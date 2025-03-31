use async_nats::SubscribeError;
use fuel_data_parser::DataParserError;
use fuel_message_broker::MessageBrokerError;
use fuel_streams_domains::{
    infra::{
        db::{DbError, SqlxError},
        record::{RecordEntityError, RecordPacketError},
        repository::RepositoryError,
    },
    SubjectsError,
};
use fuel_streams_subject::subject::SubjectPayloadError;
use fuel_web_utils::api_key::ApiKeyError;

use crate::{server::DeliverPolicyError, types::StreamResponseError};

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Unknown subject: {0}")]
    UnknownSubject(String),

    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Subscribe(#[from] SubscribeError),
    #[error(transparent)]
    DeliverPolicy(#[from] DeliverPolicyError),
    #[error(transparent)]
    MessageBrokerClient(#[from] MessageBrokerError),
    #[error(transparent)]
    RecordPacket(#[from] RecordPacketError),
    #[error(transparent)]
    Sqlx(#[from] SqlxError),
    #[error(transparent)]
    StreamResponse(#[from] StreamResponseError),
    #[error(transparent)]
    RecordEntity(#[from] RecordEntityError),
    #[error(transparent)]
    ApiKey(#[from] ApiKeyError),
    #[error(transparent)]
    JoinError(#[from] tokio::task::JoinError),
    #[error(transparent)]
    Subjects(#[from] SubjectsError),
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    SubjectPayload(#[from] SubjectPayloadError),
    #[error(transparent)]
    Encode(#[from] DataParserError),
}
