use std::fmt;

use serde::{Deserialize, Serialize};

use crate::api_key::ApiKeyError;

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiKeyRoleScope {
    Full,
    HistoricalData,
    #[default]
    LiveData,
    RestApi,
}

impl ApiKeyRoleScope {
    pub fn as_str(&self) -> &str {
        match self {
            ApiKeyRoleScope::Full => "FULL",
            ApiKeyRoleScope::HistoricalData => "HISTORICAL_DATA",
            ApiKeyRoleScope::LiveData => "LIVE_DATA",
            ApiKeyRoleScope::RestApi => "REST_API",
        }
    }

    pub fn is_full(&self) -> bool {
        matches!(self, ApiKeyRoleScope::Full)
    }

    pub fn is_historical_data(&self) -> bool {
        matches!(self, ApiKeyRoleScope::HistoricalData)
    }

    pub fn is_live_data(&self) -> bool {
        matches!(self, ApiKeyRoleScope::LiveData)
    }

    pub fn is_rest_api(&self) -> bool {
        matches!(self, ApiKeyRoleScope::RestApi)
    }
}

impl fmt::Display for ApiKeyRoleScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for ApiKeyRoleScope {
    type Error = ApiKeyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "FULL" => Ok(ApiKeyRoleScope::Full),
            "HISTORICAL_DATA" => Ok(ApiKeyRoleScope::HistoricalData),
            "LIVE_DATA" => Ok(ApiKeyRoleScope::LiveData),
            "REST_API" => Ok(ApiKeyRoleScope::RestApi),
            _ => Err(ApiKeyError::ScopePermission(value.to_string())),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ApiKeyRoleScope {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("api_scope")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApiKeyRoleScope {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        ApiKeyRoleScope::try_from(value).map_err(|e| e.to_string().into())
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ApiKeyRoleScope {
    fn encode_by_ref(
        &self,
        buf: &mut sqlx::postgres::PgArgumentBuffer,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        <&str as sqlx::Encode<sqlx::Postgres>>::encode_by_ref(
            &self.as_str(),
            buf,
        )
    }
}

// Add PgHasArrayType implementation for ApiScope
impl sqlx::postgres::PgHasArrayType for ApiKeyRoleScope {
    fn array_type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("_api_scope")
    }
}
