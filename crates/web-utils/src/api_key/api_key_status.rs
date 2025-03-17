use std::fmt;

use serde::{Deserialize, Serialize};

use super::ApiKeyError;

#[derive(
    Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Default, Hash,
)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ApiKeyStatus {
    Active,
    #[default]
    Inactive,
    Revoked,
    Expired,
}

impl ApiKeyStatus {
    pub fn as_str(&self) -> &str {
        match self {
            ApiKeyStatus::Active => "ACTIVE",
            ApiKeyStatus::Inactive => "INACTIVE",
            ApiKeyStatus::Revoked => "REVOKED",
            ApiKeyStatus::Expired => "EXPIRED",
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self, ApiKeyStatus::Active)
    }

    pub fn is_inactive(&self) -> bool {
        matches!(self, ApiKeyStatus::Inactive)
    }

    pub fn is_revoked(&self) -> bool {
        matches!(self, ApiKeyStatus::Revoked)
    }

    pub fn is_expired(&self) -> bool {
        matches!(self, ApiKeyStatus::Expired)
    }
}

impl fmt::Display for ApiKeyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<&str> for ApiKeyStatus {
    type Error = ApiKeyError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "ACTIVE" => Ok(ApiKeyStatus::Active),
            "INACTIVE" => Ok(ApiKeyStatus::Inactive),
            "REVOKED" => Ok(ApiKeyStatus::Revoked),
            "EXPIRED" => Ok(ApiKeyStatus::Expired),
            _ => Err(ApiKeyError::InvalidStatus(value.to_string())),
        }
    }
}

impl sqlx::Type<sqlx::Postgres> for ApiKeyStatus {
    fn type_info() -> sqlx::postgres::PgTypeInfo {
        sqlx::postgres::PgTypeInfo::with_name("api_key_status")
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ApiKeyStatus {
    fn decode(
        value: sqlx::postgres::PgValueRef<'r>,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let value = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        ApiKeyStatus::try_from(value).map_err(sqlx::error::BoxDynError::from)
    }
}

impl sqlx::Encode<'_, sqlx::Postgres> for ApiKeyStatus {
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
