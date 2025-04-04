use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::{FromRequestParts, Query, Request, State},
    http::{header::AUTHORIZATION, request::Parts},
    middleware::Next,
    response::Response,
};
use fuel_streams_domains::infra::Db;

use super::{
    rate_limiter::RateLimitsController,
    ApiKey,
    ApiKeyError,
    ApiKeyId,
    ApiKeyRole,
    ApiKeyValue,
    InMemoryApiKeyStorage,
    KeyStorage,
};

#[derive(Debug, Clone)]
pub struct ApiKeysManager {
    storage: Arc<InMemoryApiKeyStorage>,
    rate_limiter_controller: Arc<RateLimitsController>,
}

impl Default for ApiKeysManager {
    fn default() -> Self {
        let storage = Arc::new(InMemoryApiKeyStorage::new());
        Self {
            storage,
            rate_limiter_controller: RateLimitsController::default().arc(),
        }
    }
}

impl ApiKeysManager {
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
        Ok(db_keys)
    }

    pub async fn get_api_key_from_db(
        &self,
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
        match self.storage.find_by_key(key) {
            Ok(key) => {
                tracing::debug!("Cache hit for API key");
                Ok(key)
            }
            Err(ApiKeyError::NotFound) => {
                tracing::debug!("Cache miss for API key");
                self.get_api_key_from_db(key, db).await
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

    pub async fn extract_api_key(
        parts: &mut Parts,
    ) -> Result<ApiKeyValue, ApiKeyError> {
        if let Some(auth_header) = parts.headers.get(AUTHORIZATION) {
            let token = auth_header.to_str().map_err(|_| {
                ApiKeyError::InvalidHeader("Invalid header".to_string())
            })?;
            if token.starts_with("Bearer ") {
                return Ok(ApiKeyValue::new(
                    token.trim_start_matches("Bearer ").to_string(),
                ));
            }
        }

        let query =
            Query::<HashMap<String, String>>::from_request_parts(parts, &())
                .await
                .map_err(|_| ApiKeyError::Invalid)?;
        if let Some(key) = query.get("api_key") {
            return Ok(ApiKeyValue::new(key.clone()));
        }

        Err(ApiKeyError::NotFound)
    }

    pub async fn middleware(
        State(manager): State<Arc<Self>>,
        State(db): State<Arc<Db>>,
        req: Request,
        next: Next,
    ) -> Result<Response, ApiKeyError> {
        let mut parts = req.into_parts().0;
        let api_key_str = Self::extract_api_key(&mut parts).await?;
        let api_key = manager.validate_api_key(&api_key_str, &db).await?;
        manager.check_subscriptions(api_key.id(), api_key.role())?;
        manager.check_rate_limit(api_key.id(), api_key.role())?;
        api_key.validate_status()?;
        let mut req = Request::from_parts(parts, Body::default());
        req.extensions_mut().insert(api_key.clone());
        let response = next.run(req).await;
        Ok(response)
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
    use axum::http::{HeaderMap, HeaderValue};
    use pretty_assertions::assert_eq;

    use super::*;

    #[tokio::test]
    async fn test_key_extraction_from_header() {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str("Bearer test_api_key").unwrap(),
        );
        let req = axum::http::Request::builder()
            .uri("/test")
            .header(AUTHORIZATION, "Bearer test_api_key")
            .body(())
            .unwrap();

        let mut parts = req.into_parts().0;
        let result = ApiKeysManager::extract_api_key(&mut parts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "test_api_key");
    }

    #[tokio::test]
    async fn test_key_extraction_from_query() {
        let req = axum::http::Request::builder()
            .uri("/test?api_key=test_api_key")
            .body(())
            .unwrap();

        let mut parts = req.into_parts().0;
        let result = ApiKeysManager::extract_api_key(&mut parts).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().to_string(), "test_api_key");
    }

    #[tokio::test]
    async fn test_key_extraction_missing() {
        let req = axum::http::Request::builder()
            .uri("/test")
            .body(())
            .unwrap();

        let mut parts = req.into_parts().0;
        let result = ApiKeysManager::extract_api_key(&mut parts).await;
        assert!(result.is_err());
        assert!(matches!(result, Err(ApiKeyError::NotFound)));
    }
}
