use std::{collections::HashMap, sync::Arc};

use actix_web::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use fuel_streams_store::db::Db;
use serde::{Deserialize, Serialize};

use super::{ApiKeyError, ApiKeyStorageError};
use crate::server::middlewares::api_key::{
    ApiKey,
    InMemoryApiKeyStorage,
    KeyStorage,
};

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct DbUserApiKey {
    pub user_id: i64,
    pub user_name: String,
    pub api_key: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiKeyManagerError {
    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Invalid API key")]
    InvalidApiKey,
}

const BEARER: &str = "Bearer";

#[derive(Debug, Clone)]
pub struct ApiKeysManager {
    pub db: Arc<Db>,
    pub storage: Arc<InMemoryApiKeyStorage>,
}

impl ApiKeysManager {
    pub fn new(db: &Arc<Db>) -> Self {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        Self {
            db: db.to_owned(),
            storage,
        }
    }

    pub async fn load_from_db(&self) -> Result<Vec<ApiKey>, ApiKeyError> {
        let db_records =
            sqlx::query_as::<_, DbUserApiKey>("SELECT * FROM api_keys")
                .fetch_all(&self.db.pool)
                .await
                .map_err(ApiKeyManagerError::DatabaseError)?;
        let keys = db_records
            .into_iter()
            .map(|record| {
                ApiKey::new(
                    record.user_id as u64,
                    record.user_name,
                    record.api_key,
                )
            })
            .collect::<Vec<ApiKey>>();
        Ok(keys)
    }

    pub async fn get_api_key_from_db(
        self,
        api_key: &str,
    ) -> Result<Option<ApiKey>, ApiKeyError> {
        let record = sqlx::query_as::<_, DbUserApiKey>(
            "SELECT * FROM api_keys WHERE api_key = $1",
        )
        .bind(api_key)
        .fetch_optional(&self.db.pool)
        .await
        .map_err(ApiKeyManagerError::DatabaseError)?;
        Ok(record
            .map(|r| ApiKey::new(r.user_id as u64, r.user_name, r.api_key)))
    }

    pub async fn validate_api_key(
        &self,
        api_key: &str,
    ) -> Result<Option<ApiKey>, ApiKeyError> {
        // First try in-memory cache
        match self.storage.find_by_key(api_key) {
            Ok(key) => {
                tracing::debug!("Cache hit for API key");
                Ok(Some(key))
            }
            Err(ApiKeyError::Storage(ApiKeyStorageError::KeyNotFound)) => {
                tracing::debug!("Cache miss for API key");
                // If not in memory, try loading from DB
                let key = self.clone().get_api_key_from_db(api_key).await?;
                if let Some(ref key) = key {
                    // Store in memory for future use
                    if let Err(e) = self.storage.insert(key) {
                        tracing::warn!("Failed to cache API key: {}", e);
                    }
                }
                Ok(key)
            }
            Err(e) => Err(e),
        }
    }

    pub fn key_from_headers(
        &self,
        (headers, query_map): (HeaderMap, HashMap<String, String>),
    ) -> Result<String, ApiKeyError> {
        // Add API key from query params to headers if present
        let mut headers = headers;
        for (key, value) in query_map.iter() {
            if key.eq_ignore_ascii_case("api_key") {
                let token = format!("Bearer {}", value);
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&token)
                        .map_err(ApiKeyError::InvalidHeader)?,
                );
            }
        }

        match Self::from_query_string(&headers) {
            Ok(key) => Ok(key.to_string()),
            Err(_) => Err(ApiKeyError::NotFound),
        }
    }

    fn from_query_string(headers: &HeaderMap) -> Result<String, ApiKeyError> {
        let token = headers.get(AUTHORIZATION).ok_or(ApiKeyError::NotFound)?;
        let token = match token.to_str() {
            Ok(token) => token,
            Err(_) => return Err(ApiKeyError::Invalid),
        };

        if !token.starts_with(BEARER) {
            return Err(ApiKeyError::Invalid);
        }
        urlencoding::decode(token.trim_start_matches(BEARER))
            .map_err(|_| ApiKeyError::Invalid)
            .map(|decoded| decoded.trim().to_string())
    }
}
