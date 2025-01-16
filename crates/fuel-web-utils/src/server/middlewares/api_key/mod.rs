pub mod transform;
pub mod validator;

use std::{collections::HashMap, fmt};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ApiKeyLimit {
    Limited(u32),
    Unlimited,
}

impl fmt::Display for ApiKeyLimit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiKeyLimit::Limited(limit) => write!(f, "{}", limit),
            ApiKeyLimit::Unlimited => write!(f, "unlimited"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKeyLimits {
    pub max_reads_per_minute: ApiKeyLimit,
    pub max_writes_per_minute: ApiKeyLimit,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKeyRestrictions {
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub enum ApiKeyStatus {
    Active,
    Inactive,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiKey {
    pub user_id: uuid::Uuid,
    pub key: String,
    pub limits: ApiKeyLimits,
    pub restrictions: ApiKeyRestrictions,
    pub status: ApiKeyStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct InMemoryApiKeyStorage {
    map: HashMap<String, ApiKey>,
}

impl InMemoryApiKeyStorage {
    pub fn new() -> Self {
        Self {
            map: HashMap::default(),
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
    fn store_api_key_for_user(
        &mut self,
        user_id: &str,
        value: &ApiKey,
    ) -> Result<ApiKey, ApiKeyStorageError>;
    fn retrieve_api_key_for_user(
        &self,
        user_id: &str,
    ) -> Result<ApiKey, ApiKeyStorageError>;
    fn delete_api_key_for_user(
        &mut self,
        user_id: &str,
    ) -> Result<bool, ApiKeyStorageError>;
    fn find_api_key(&self, api_key: &str)
        -> Result<ApiKey, ApiKeyStorageError>;
}

impl KeyStorage for InMemoryApiKeyStorage {
    fn store_api_key_for_user(
        &mut self,
        user_id: &str,
        value: &ApiKey,
    ) -> Result<ApiKey, ApiKeyStorageError> {
        let entry = self.map.insert(user_id.to_string(), value.clone());
        if entry.is_none() {
            return Err(ApiKeyStorageError::KeyAlreadyExists);
        }
        Ok(entry.unwrap())
    }

    fn retrieve_api_key_for_user(
        &self,
        user_id: &str,
    ) -> Result<ApiKey, ApiKeyStorageError> {
        match self.map.get(user_id).cloned() {
            Some(api_key) => Ok(api_key),
            None => Err(ApiKeyStorageError::KeyNotFound),
        }
    }

    fn delete_api_key_for_user(
        &mut self,
        user_id: &str,
    ) -> Result<bool, ApiKeyStorageError> {
        if !self.map.contains_key(user_id) {
            return Err(ApiKeyStorageError::KeyNotFound);
        }
        Ok(self.map.remove(user_id).is_some())
    }

    fn find_api_key(
        &self,
        api_key: &str,
    ) -> Result<ApiKey, ApiKeyStorageError> {
        self.map
            .values()
            .find(|v| v.key == api_key)
            .cloned()
            .ok_or(ApiKeyStorageError::KeyNotFound)
    }
}
