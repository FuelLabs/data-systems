use dashmap::DashMap;

use super::{ApiKey, ApiKeyError};

#[derive(Debug, Default)]
pub struct InMemoryApiKeyStorage {
    map: DashMap<String, ApiKey>,
}

impl InMemoryApiKeyStorage {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }
}

impl Clone for InMemoryApiKeyStorage {
    fn clone(&self) -> Self {
        Self {
            map: self.map.clone(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyStorageError {
    #[error("Key not found")]
    KeyNotFound,
    #[error("Key already exists")]
    KeyAlreadyExists,
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub trait KeyStorage {
    fn insert(&self, api_key: &ApiKey) -> Result<ApiKey, ApiKeyError>;
    fn retrieve(&self, api_key: &ApiKey) -> Result<ApiKey, ApiKeyError>;
    fn delete(&self, api_key: &ApiKey) -> Result<bool, ApiKeyError>;
    fn find_by_key(&self, value: &str) -> Result<ApiKey, ApiKeyError>;
}

impl KeyStorage for InMemoryApiKeyStorage {
    fn insert(&self, api_key: &ApiKey) -> Result<ApiKey, ApiKeyError> {
        let storage_key = api_key.storage_key();
        match self.map.get(&storage_key) {
            Some(_) => Err(ApiKeyStorageError::KeyAlreadyExists.into()),
            None => {
                self.map.insert(storage_key, api_key.clone());
                Ok(api_key.clone())
            }
        }
    }

    fn retrieve(&self, api_key: &ApiKey) -> Result<ApiKey, ApiKeyError> {
        self.map
            .get(&api_key.storage_key())
            .map(|r| r.clone())
            .ok_or(ApiKeyStorageError::KeyNotFound.into())
    }

    fn delete(&self, api_key: &ApiKey) -> Result<bool, ApiKeyError> {
        let storage_key = api_key.storage_key();
        if !self.map.contains_key(&storage_key) {
            return Err(ApiKeyStorageError::KeyNotFound.into());
        }
        self.map.remove(&storage_key);
        Ok(true)
    }

    fn find_by_key(&self, value: &str) -> Result<ApiKey, ApiKeyError> {
        self.map
            .iter()
            .find(|r| r.value().key().as_ref() == value)
            .map(|r| r.value().clone())
            .ok_or(ApiKeyStorageError::KeyNotFound.into())
    }
}
