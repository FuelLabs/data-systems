use std::fmt;

use axum::http::Request;
use rand::{distr::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

use super::{
    ApiKeyError,
    ApiKeyId,
    ApiKeyRole,
    ApiKeyRoleId,
    ApiKeyRoleName,
    ApiKeyRoleScope,
    ApiKeyStatus,
    ApiKeyUserName,
    ApiKeyValue,
    RateLimitPerMinute,
    SubscriptionCount,
};

#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct DbApiKey {
    id: ApiKeyId,
    user_name: ApiKeyUserName,
    api_key: ApiKeyValue,
    role_id: ApiKeyRoleId,
    status: ApiKeyStatus,
}

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    sqlx::FromRow,
    Hash,
    utoipa::ToSchema,
)]
pub struct ApiKey {
    id: ApiKeyId,
    user_name: ApiKeyUserName,
    api_key: ApiKeyValue,
    role: ApiKeyRole,
    status: ApiKeyStatus,
}

impl ApiKey {
    pub fn new(
        id: ApiKeyId,
        user_name: ApiKeyUserName,
        api_key: ApiKeyValue,
        role: ApiKeyRole,
        status: ApiKeyStatus,
    ) -> Self {
        Self {
            id,
            user_name,
            api_key,
            role,
            status,
        }
    }

    pub fn with_role(self, role: ApiKeyRole) -> Self {
        Self { role, ..self }
    }

    pub fn from_req<B>(req: &Request<B>) -> Result<ApiKey, ApiKeyError> {
        match req.extensions().get::<ApiKey>() {
            Some(api_key) => {
                tracing::info!(
                    id = %api_key.id,
                    user_name = %api_key.user_name,
                    "Authenticated request"
                );
                Ok(api_key.to_owned())
            }
            None => {
                tracing::warn!("Unauthenticated request attempt");
                Err(ApiKeyError::NotFound)
            }
        }
    }

    pub fn id(&self) -> &ApiKeyId {
        &self.id
    }

    pub fn user(&self) -> &ApiKeyUserName {
        &self.user_name
    }

    pub fn key(&self) -> &ApiKeyValue {
        &self.api_key
    }

    pub fn role(&self) -> &ApiKeyRole {
        &self.role
    }

    pub fn status(&self) -> &ApiKeyStatus {
        &self.status
    }

    pub fn storage_key(&self) -> String {
        format!("{}-{}", self.id(), self.user())
    }

    pub fn generate_random_api_key() -> String {
        let random_num = rand::rng()
            .sample_iter(&Alphanumeric)
            .filter(|c| c.is_ascii_alphabetic())
            .take(32)
            .map(char::from)
            .collect::<String>();
        format!("fuel-{}", random_num)
    }

    pub fn as_str(&self) -> String {
        format!(
            "ApiKey {{ id: {}, user: {}, key: {}, status: {:?} }}",
            self.id, self.user_name, self.api_key, self.status
        )
    }

    pub async fn create(
        pool: &sqlx::PgPool,
        user_name: &ApiKeyUserName,
        role_name: &ApiKeyRoleName,
    ) -> Result<Self, ApiKeyError> {
        let api_key_value = ApiKeyValue::new(Self::generate_random_api_key());
        let role =
            ApiKeyRole::fetch_by_name(pool, role_name)
                .await
                .map_err(|e| match e {
                    sqlx::Error::RowNotFound => {
                        ApiKeyError::RolePermission(role_name.to_string())
                    }
                    _ => ApiKeyError::DatabaseError(e.to_string()),
                })?;

        let db_record = sqlx::query_as::<_, DbApiKey>(
            "INSERT INTO api_keys (user_name, api_key, status, role_id)
             VALUES ($1, $2, 'ACTIVE', $3)
             RETURNING id, user_name, api_key, status, role_id",
        )
        .bind(user_name)
        .bind(&api_key_value)
        .bind(role.id())
        .fetch_one(pool)
        .await
        .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        Ok(ApiKey::from((db_record, role)))
    }

    pub async fn update_status(
        pool: &sqlx::PgPool,
        key: &ApiKeyValue,
        status: ApiKeyStatus,
    ) -> Result<Self, ApiKeyError> {
        let db_record = sqlx::query_as::<_, DbApiKey>(
            "UPDATE api_keys
             SET status = $1
             WHERE api_key = $2
             RETURNING id, user_name, api_key, status, role_id",
        )
        .bind(&status)
        .bind(key)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiKeyError::NotFound,
            _ => ApiKeyError::DatabaseError(e.to_string()),
        })?;

        let role = ApiKeyRole::fetch_by_id(pool, db_record.role_id)
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        Ok(ApiKey::from((db_record, role)))
    }

    pub async fn fetch_by_key(
        pool: &sqlx::PgPool,
        key: &ApiKeyValue,
    ) -> Result<Self, ApiKeyError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        let db_record = sqlx::query_as::<_, DbApiKey>(
            "SELECT id, user_name, api_key, role_id, status
             FROM api_keys
             WHERE api_key = $1",
        )
        .bind(key)
        .fetch_one(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ApiKeyError::NotFound,
            _ => ApiKeyError::DatabaseError(e.to_string()),
        })?;

        let role = ApiKeyRole::fetch_by_id(&mut *tx, db_record.role_id)
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        Ok(ApiKey::from((db_record, role)))
    }

    pub async fn fetch_all(
        pool: &sqlx::PgPool,
    ) -> Result<Vec<Self>, ApiKeyError> {
        let mut tx = pool
            .begin()
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        let db_records = sqlx::query_as::<_, DbApiKey>(
            "SELECT id, user_name, api_key, role_id, status
             FROM api_keys
             ORDER BY id",
        )
        .fetch_all(&mut *tx)
        .await
        .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        let mut api_keys = Vec::with_capacity(db_records.len());
        for db_record in db_records {
            let role = ApiKeyRole::fetch_by_id(&mut *tx, db_record.role_id)
                .await
                .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;
            api_keys.push(ApiKey::from((db_record, role)));
        }

        tx.commit()
            .await
            .map_err(|e| ApiKeyError::DatabaseError(e.to_string()))?;

        Ok(api_keys)
    }

    pub fn validate_status(&self) -> Result<(), ApiKeyError> {
        let status = self.status.as_str();
        match self.status {
            ApiKeyStatus::Active => Ok(()),
            ApiKeyStatus::Inactive => {
                Err(ApiKeyError::BadStatus(status.to_string()))
            }
            ApiKeyStatus::Revoked => {
                Err(ApiKeyError::BadStatus(status.to_string()))
            }
            ApiKeyStatus::Expired => {
                Err(ApiKeyError::BadStatus(status.to_string()))
            }
        }
    }

    pub fn subscription_limit(&self) -> Option<SubscriptionCount> {
        self.role.subscription_limit()
    }

    pub fn scopes(&self) -> Vec<ApiKeyRoleScope> {
        self.role.scopes()
    }

    pub fn rate_limit_per_minute(&self) -> Option<RateLimitPerMinute> {
        self.role.rate_limit_per_minute()
    }
}

impl From<(DbApiKey, ApiKeyRole)> for ApiKey {
    fn from((db_record, role): (DbApiKey, ApiKeyRole)) -> Self {
        Self::new(
            db_record.id,
            db_record.user_name,
            db_record.api_key,
            role,
            db_record.status,
        )
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub struct MockApiKey(pub ApiKey);
#[cfg(any(test, feature = "test-helpers"))]
impl MockApiKey {
    pub fn new(api_key: ApiKey) -> Self {
        Self(api_key)
    }

    pub fn into_inner(self) -> ApiKey {
        self.0
    }

    pub fn admin(id: ApiKeyId) -> Self {
        use super::role::MockApiKeyRole;
        let api_key = ApiKey::new(
            id,
            "admin".into(),
            "fuel-admin-key".into(),
            MockApiKeyRole::admin().into_inner(),
            ApiKeyStatus::Active,
        );
        Self(api_key)
    }

    pub fn builder(id: ApiKeyId) -> Self {
        use super::role::MockApiKeyRole;
        let api_key = ApiKey::new(
            id,
            "builder".into(),
            "fuel-builder-key".into(),
            MockApiKeyRole::builder().into_inner(),
            ApiKeyStatus::Active,
        );
        Self(api_key)
    }

    pub fn web_client(id: ApiKeyId) -> Self {
        use super::role::MockApiKeyRole;
        let api_key = ApiKey::new(
            id,
            "web_client".into(),
            "fuel-web-client-key".into(),
            MockApiKeyRole::web_client().into_inner(),
            ApiKeyStatus::Active,
        );
        Self(api_key)
    }
}
