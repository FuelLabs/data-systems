use fuel_streams_types::BlockHeight;
use serde::{Deserialize, Serialize};

use crate::api_key::{
    ApiKeyError,
    ApiKeyRoleId,
    ApiKeyRoleName,
    ApiKeyRoleScope,
    HistoricalLimit,
    RateLimitPerMinute,
    SubscriptionCount,
};

#[derive(
    Debug,
    Clone,
    Serialize,
    Deserialize,
    PartialEq,
    Eq,
    sqlx::FromRow,
    Default,
    Hash,
)]
pub struct ApiKeyRole {
    id: ApiKeyRoleId,
    name: ApiKeyRoleName,
    scopes: Vec<ApiKeyRoleScope>,
    subscription_limit: Option<SubscriptionCount>,
    rate_limit_per_minute: Option<RateLimitPerMinute>,
    historical_limit: Option<HistoricalLimit>,
}

impl ApiKeyRole {
    pub fn new(
        id: ApiKeyRoleId,
        name: ApiKeyRoleName,
        scopes: Vec<ApiKeyRoleScope>,
        subscription_limit: Option<SubscriptionCount>,
        rate_limit_per_minute: Option<RateLimitPerMinute>,
        historical_limit: Option<HistoricalLimit>,
    ) -> Self {
        Self {
            id,
            name,
            scopes,
            subscription_limit,
            rate_limit_per_minute,
            historical_limit,
        }
    }

    pub fn id(&self) -> &ApiKeyRoleId {
        &self.id
    }

    pub fn name(&self) -> &ApiKeyRoleName {
        &self.name
    }

    pub fn subscription_limit(&self) -> Option<SubscriptionCount> {
        self.subscription_limit
    }

    pub fn scopes(&self) -> Vec<ApiKeyRoleScope> {
        self.scopes.to_vec()
    }

    pub fn rate_limit_per_minute(&self) -> Option<RateLimitPerMinute> {
        self.rate_limit_per_minute
    }

    pub fn historical_limit(&self) -> Option<HistoricalLimit> {
        self.historical_limit
    }

    pub fn has_scopes(
        &self,
        scopes: &[ApiKeyRoleScope],
    ) -> Result<(), ApiKeyError> {
        if self.scopes.iter().any(|s| scopes.contains(s)) {
            Ok(())
        } else {
            let scopes_str = self
                .scopes
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>()
                .join(", ");
            Err(ApiKeyError::ScopePermission(scopes_str))
        }
    }

    pub async fn fetch_all<'e, 'c, E>(
        executor: E,
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        'c: 'e,
        E: sqlx::PgExecutor<'c>,
    {
        sqlx::query_as::<_, Self>(
            "SELECT id, name, scopes, subscription_limit, rate_limit_per_minute, historical_limit
             FROM api_key_roles
             ORDER BY name",
        )
        .fetch_all(executor)
        .await
    }

    pub async fn fetch_by_name<'e, 'c, E>(
        executor: E,
        name: &ApiKeyRoleName,
    ) -> Result<Self, sqlx::Error>
    where
        'c: 'e,
        E: sqlx::PgExecutor<'c>,
    {
        sqlx::query_as::<_, Self>(
            "SELECT id, name, scopes, subscription_limit, rate_limit_per_minute, historical_limit
             FROM api_key_roles
             WHERE name = $1::api_role",
        )
        .bind(name)
        .fetch_one(executor)
        .await
    }

    pub async fn fetch_by_id<'e, 'c, E>(
        executor: E,
        id: ApiKeyRoleId,
    ) -> Result<Self, sqlx::Error>
    where
        'c: 'e,
        E: sqlx::PgExecutor<'c>,
    {
        sqlx::query_as::<_, Self>(
            "SELECT id, name, scopes, subscription_limit, rate_limit_per_minute, historical_limit
             FROM api_key_roles
             WHERE id = $1",
        )
        .bind(id)
        .fetch_one(executor)
        .await
    }

    pub fn validate_subscription_limit(
        &self,
        current_count: SubscriptionCount,
    ) -> Result<SubscriptionCount, ApiKeyError> {
        let has_data_scopes = self
            .scopes
            .iter()
            .any(|s| s.is_historical_data() || s.is_live_data());

        if !has_data_scopes {
            return Ok(current_count)
        }

        if let Some(limit) = self.subscription_limit() {
            if current_count > limit {
                return Err(ApiKeyError::SubscriptionLimitExceeded(
                    limit.to_string(),
                ));
            }
        }
        Ok(current_count)
    }

    pub fn validate_rate_limit(
        &self,
        current_count: RateLimitPerMinute,
    ) -> Result<RateLimitPerMinute, ApiKeyError> {
        if let Some(limit) = self.rate_limit_per_minute() {
            if current_count > limit {
                return Err(ApiKeyError::RateLimitExceeded(limit.to_string()));
            }
        }
        Ok(current_count)
    }

    pub fn validate_historical_limit(
        &self,
        last_height: BlockHeight,
        current_height: BlockHeight,
    ) -> Result<(), ApiKeyError> {
        if let Some(limit) = self.historical_limit() {
            let diff = last_height.into_inner() - current_height.into_inner();
            if diff > limit.into_inner() as u64 {
                return Err(ApiKeyError::HistoricalLimitExceeded(
                    limit.to_string(),
                ));
            }
        }
        Ok(())
    }
}

impl sqlx::Type<sqlx::Postgres> for ApiKeyRole {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("api_key_role")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApiKeyRole {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let (
            id,
            name,
            scopes,
            subscription_limit,
            rate_limit_per_minute,
            historical_limit,
        ) = <(
            ApiKeyRoleId,
            ApiKeyRoleName,
            Vec<ApiKeyRoleScope>,
            Option<SubscriptionCount>,
            Option<RateLimitPerMinute>,
            Option<HistoricalLimit>,
        )>::decode(value)?;
        Ok(Self {
            id,
            name,
            scopes,
            subscription_limit,
            rate_limit_per_minute,
            historical_limit,
        })
    }
}

#[cfg(any(test, feature = "test-helpers"))]
pub struct MockApiKeyRole(ApiKeyRole);
#[cfg(any(test, feature = "test-helpers"))]
impl MockApiKeyRole {
    pub fn new(role: ApiKeyRole) -> Self {
        Self(role)
    }

    pub fn into_inner(self) -> ApiKeyRole {
        self.0
    }

    pub fn admin() -> Self {
        Self(ApiKeyRole::new(
            ApiKeyRoleId::from(1),
            ApiKeyRoleName::Admin,
            vec![
                ApiKeyRoleScope::ManageApiKeys,
                ApiKeyRoleScope::HistoricalData,
                ApiKeyRoleScope::LiveData,
                ApiKeyRoleScope::RestApi,
            ],
            None,
            None,
            None,
        ))
    }

    pub fn amm() -> Self {
        Self(ApiKeyRole::new(
            ApiKeyRoleId::from(2),
            ApiKeyRoleName::Amm,
            vec![
                ApiKeyRoleScope::HistoricalData,
                ApiKeyRoleScope::LiveData,
                ApiKeyRoleScope::RestApi,
            ],
            None,
            None,
            None,
        ))
    }

    pub fn builder() -> Self {
        Self(ApiKeyRole::new(
            ApiKeyRoleId::from(3),
            ApiKeyRoleName::Builder,
            vec![
                ApiKeyRoleScope::HistoricalData,
                ApiKeyRoleScope::LiveData,
                ApiKeyRoleScope::RestApi,
            ],
            Some(SubscriptionCount::from(50)),
            Some(RateLimitPerMinute::from(7)),
            Some(HistoricalLimit::from(600)),
        ))
    }

    pub fn web_client() -> Self {
        Self(ApiKeyRole::new(
            ApiKeyRoleId::from(4),
            ApiKeyRoleName::WebClient,
            vec![ApiKeyRoleScope::LiveData, ApiKeyRoleScope::RestApi],
            None,
            Some(RateLimitPerMinute::from(1000)),
            None,
        ))
    }

    pub fn no_scopes() -> Self {
        Self(ApiKeyRole::new(
            ApiKeyRoleId::from(5),
            ApiKeyRoleName::WebClient,
            vec![],
            None,
            None,
            None,
        ))
    }
}
