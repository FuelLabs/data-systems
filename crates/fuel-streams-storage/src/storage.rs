use async_trait::async_trait;
use thiserror::Error;

use crate::StorageConfig;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Failed to store object: {0}")]
    StoreError(String),
    #[error("Failed to retrieve object: {0}")]
    RetrieveError(String),
    #[error("Failed to delete object: {0}")]
    DeleteError(String),
    #[error("Failed to initialize storage: {0}")]
    InitError(String),
}

#[async_trait]
pub trait Storage: std::fmt::Debug + Send + Sync {
    type Config: StorageConfig;

    async fn new(config: Self::Config) -> Result<Self, StorageError>
    where
        Self: Sized;

    async fn new_admin() -> Result<Self, StorageError>
    where
        Self: Sized,
    {
        Self::new(Self::Config::admin_opts()).await
    }

    async fn new_public() -> Result<Self, StorageError>
    where
        Self: Sized,
    {
        Self::new(Self::Config::public_opts()).await
    }

    async fn store(&self, key: &str, data: Vec<u8>)
        -> Result<(), StorageError>;

    async fn retrieve(&self, key: &str) -> Result<Vec<u8>, StorageError>;

    async fn delete(&self, key: &str) -> Result<(), StorageError>;
}
