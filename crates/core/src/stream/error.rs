use async_nats::SubscribeError;
use fuel_message_broker::MessageBrokerError;
use fuel_streams_store::{
    db::{DbError, SqlxError},
    record::RecordPacketError,
    store::StoreError,
};

use crate::server::DeliverPolicyError;

#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    Db(#[from] DbError),
    #[error(transparent)]
    Store(#[from] StoreError),
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
}
