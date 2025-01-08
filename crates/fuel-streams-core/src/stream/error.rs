use async_nats::{client::PublishErrorKind, SubscribeError};
use fuel_streams_store::{db::DbError, store::StoreError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Subscribe(#[from] SubscribeError),
    #[error(transparent)]
    Nats(#[from] async_nats::error::Error<PublishErrorKind>),
    #[error(transparent)]
    Db(#[from] DbError),
}
