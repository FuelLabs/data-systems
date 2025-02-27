use std::{collections::HashMap, sync::Arc};

use actix_web::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use fuel_streams_store::db::Db;

use super::{
    rate_limiter::RateLimitsController,
    ApiKeyError,
    ApiKeyId,
    ApiKeyRole,
    ApiKeyStorageError,
    ApiKeyValue,
};
use crate::api_key::{ApiKey, InMemoryApiKeyStorage, KeyStorage};

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
    storage: Arc<InMemoryApiKeyStorage>,
    rate_limiter_controller: Arc<RateLimitsController>,
}

impl Default for ApiKeysManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiKeysManager {
    pub fn new() -> Self {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        Self {
            storage,
            rate_limiter_controller: RateLimitsController::default().arc(),
        }
    }

    pub fn storage(&self) -> &Arc<InMemoryApiKeyStorage> {
        &self.storage
    }

    pub fn rate_limiter(&self) -> &Arc<RateLimitsController> {
        &self.rate_limiter_controller
    }

    pub async fn load_from_db(
        &self,
        db: &Arc<Db>,
    ) -> Result<Vec<ApiKey>, ApiKeyError> {
        let pool = db.pool_ref();
        let db_keys = ApiKey::fetch_all(pool).await?;
        let api_keys = db_keys.into_iter().collect();
        Ok(api_keys)
    }

    pub async fn get_api_key_from_db(
        self,
        key: &ApiKeyValue,
        db: &Arc<Db>,
    ) -> Result<ApiKey, ApiKeyError> {
        let pool = db.pool_ref();
        let api_key = ApiKey::fetch_by_key(pool, key).await?;
        Ok(api_key)
    }

    pub async fn validate_api_key(
        &self,
        key: &ApiKeyValue,
        db: &Arc<Db>,
    ) -> Result<ApiKey, ApiKeyError> {
        // First try in-memory cache
        match self.storage.find_by_key(key) {
            Ok(key) => {
                tracing::debug!("Cache hit for API key");
                Ok(key)
            }
            Err(ApiKeyError::Storage(ApiKeyStorageError::KeyNotFound)) => {
                tracing::debug!("Cache miss for API key");
                self.clone().get_api_key_from_db(key, db).await
            }
            Err(e) => Err(e),
        }
    }

    pub fn check_subscriptions(
        &self,
        id: &ApiKeyId,
        role: &ApiKeyRole,
    ) -> Result<(), ApiKeyError> {
        let (allowed, limit) =
            self.rate_limiter().check_subscriptions(id, role)?;
        if !allowed {
            return Err(ApiKeyError::RateLimitExceeded(limit.to_string()));
        }
        Ok(())
    }

    pub fn check_rate_limit(
        &self,
        id: &ApiKeyId,
        role: &ApiKeyRole,
    ) -> Result<(), ApiKeyError> {
        let (allowed, limit) =
            self.rate_limiter().check_rate_limit(id, role)?;
        if !allowed {
            return Err(ApiKeyError::RateLimitExceeded(limit.to_string()));
        }
        Ok(())
    }

    pub fn key_from_headers(
        &self,
        (headers, query_map): (HeaderMap, HashMap<String, String>),
    ) -> Result<ApiKeyValue, ApiKeyError> {
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
            Ok(key) => Ok(key),
            Err(_) => Err(ApiKeyError::NotFound),
        }
    }

    fn from_query_string(
        headers: &HeaderMap,
    ) -> Result<ApiKeyValue, ApiKeyError> {
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
            .map(|decoded| decoded.trim().to_string().into())
    }

    #[cfg(any(test, feature = "test-helpers"))]
    pub fn new_for_testing() -> Self {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        let rate_limiter = RateLimitsController::default().arc();
        Self {
            storage,
            rate_limiter_controller: rate_limiter,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use actix_web::http::header::{HeaderMap, HeaderValue, AUTHORIZATION};
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_key_from_headers_with_authorization_header() {
        let manager = ApiKeysManager::new_for_testing();

        // Create headers with Authorization
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer test_api_key").unwrap(),
        );

        // Empty query params
        let query_map = HashMap::new();

        // Test extraction
        let result = manager.key_from_headers((headers, query_map));
        assert!(
            result.is_ok(),
            "Should extract API key from Authorization header"
        );
        assert_eq!(result.unwrap().to_string(), "test_api_key");
    }

    #[test]
    fn test_key_from_headers_with_query_param() {
        let manager = ApiKeysManager::new_for_testing();

        // Empty headers
        let headers = HeaderMap::new();

        // Query params with api_key
        let mut query_map = HashMap::new();
        query_map.insert("api_key".to_string(), "test_api_key".to_string());

        // Test extraction
        let result = manager.key_from_headers((headers, query_map));
        assert!(
            result.is_ok(),
            "Should extract API key from query parameters"
        );
        assert_eq!(result.unwrap().to_string(), "test_api_key");
    }

    #[test]
    fn test_key_from_headers_missing_key() {
        let manager = ApiKeysManager::new_for_testing();

        // Empty headers
        let headers = HeaderMap::new();

        // Empty query params
        let query_map = HashMap::new();

        // Test extraction
        let result = manager.key_from_headers((headers, query_map));
        assert!(result.is_err(), "Should fail when no API key is provided");
        assert!(matches!(result, Err(ApiKeyError::NotFound)));
    }

    #[test]
    fn test_key_from_headers_invalid_format() {
        let manager = ApiKeysManager::new_for_testing();

        // Create headers with invalid Authorization format
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Basic test_api_key").unwrap(),
        );

        // Empty query params
        let query_map = HashMap::new();

        // Test extraction
        let result = manager.key_from_headers((headers, query_map));
        assert!(
            result.is_err(),
            "Should fail with invalid Authorization format"
        );
    }
}
