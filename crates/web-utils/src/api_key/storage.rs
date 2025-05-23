use std::time::{Duration, Instant};

use dashmap::DashMap;

use super::{ApiKey, ApiKeyError};

#[derive(Debug, Default)]
pub struct InMemoryApiKeyStorage {
    map: DashMap<String, (ApiKey, Instant)>,
}

impl InMemoryApiKeyStorage {
    pub fn new() -> Self {
        Self {
            map: DashMap::new(),
        }
    }

    fn is_expired(instant: &Instant) -> bool {
        instant.elapsed() > Duration::from_secs(600) // 10 minutes
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
                self.map
                    .insert(storage_key, (api_key.clone(), Instant::now()));
                Ok(api_key.clone())
            }
        }
    }

    fn retrieve(&self, api_key: &ApiKey) -> Result<ApiKey, ApiKeyError> {
        let storage_key = api_key.storage_key();
        self.find_by_key(&storage_key)
    }

    fn delete(&self, api_key: &ApiKey) -> Result<bool, ApiKeyError> {
        let storage_key = api_key.storage_key();
        if !self.map.contains_key(&storage_key) {
            return Err(ApiKeyError::NotFound);
        }
        self.map.remove(&storage_key);
        Ok(true)
    }

    fn find_by_key(&self, value: &str) -> Result<ApiKey, ApiKeyError> {
        let value = value.to_string();
        // We need to create a block here to avoid deadlocks with DashMap
        let api_key_result = {
            match self.map.get(&value) {
                Some(entry) => {
                    let (api_key, timestamp) = &*entry;
                    if !Self::is_expired(timestamp) {
                        Some(api_key.clone())
                    } else {
                        None
                    }
                }
                None => None,
            }
        };
        match api_key_result {
            Some(api_key) => Ok(api_key),
            None => {
                self.map.remove(&value);
                Err(ApiKeyError::NotFound)
            }
        }
    }
}
